#include "fat16.h"
#include "ata.h"
#include "string.h"
#include "vga.h"

#define SECTOR_SIZE           512
#define FAT16_PARTITION_START 2048

typedef struct {
    uint8_t  jmp[3];
    char     oem_name[8];
    uint16_t bytes_per_sector;
    uint8_t  sectors_per_cluster;
    uint16_t reserved_sectors;
    uint8_t  num_fats;
    uint16_t root_entries;
    uint16_t total_sectors_16;
    uint8_t  media_type;
    uint16_t fat_size_16;
    uint16_t sectors_per_track;
    uint16_t num_heads;
    uint32_t hidden_sectors;
    uint32_t total_sectors_32;
    uint8_t  drive_number;
    uint8_t  reserved1;
    uint8_t  boot_signature;
    uint32_t volume_id;
    char     volume_label[11];
    char     fs_type[8];
} __attribute__((packed)) fat16_bpb_t;

static fat16_bpb_t bpb;
static uint32_t fat_start, root_dir_start, data_start, root_dir_sectors;
static uint8_t  sec_buf[SECTOR_SIZE];
static uint8_t  sec_buf2[SECTOR_SIZE];
static uint8_t  fat_buf[SECTOR_SIZE];
static uint32_t cached_fat_sec = 0xFFFFFFFF;
static uint8_t  fat_dirty = 0;

//========
static int  rd(uint32_t lba, void* b)       { return ata_read_sector (FAT16_PARTITION_START+lba, b); }
static int  wr(uint32_t lba, const void* b) { return ata_write_sector(FAT16_PARTITION_START+lba, b); }

static uint32_t clus_lba(uint16_t c) {
    return data_start + (uint32_t)(c-2)*bpb.sectors_per_cluster;
}

// FAT cache

static uint16_t fat_get(uint16_t c) {
    uint32_t sec = (c*2)/SECTOR_SIZE, off = (c*2)%SECTOR_SIZE;
    if (sec != cached_fat_sec) {
        if (fat_dirty) { wr(fat_start+cached_fat_sec,fat_buf); fat_dirty=0; }
        rd(fat_start+sec, fat_buf);
        cached_fat_sec = sec;
    }
    return *(uint16_t*)(fat_buf+off);
}

static void fat_set(uint16_t c, uint16_t v) {
    uint32_t sec = (c*2)/SECTOR_SIZE, off = (c*2)%SECTOR_SIZE;
    if (sec != cached_fat_sec) {
        if (fat_dirty) { wr(fat_start+cached_fat_sec,fat_buf); fat_dirty=0; }
        rd(fat_start+sec, fat_buf);
        cached_fat_sec = sec;
    }
    *(uint16_t*)(fat_buf+off) = v;
    fat_dirty = 1;
}

static void fat_flush(void) {
    if (fat_dirty && cached_fat_sec != 0xFFFFFFFF) {
        wr(fat_start+cached_fat_sec, fat_buf);
        wr(fat_start+bpb.fat_size_16+cached_fat_sec, fat_buf);
        fat_dirty = 0;
    }
}

static uint16_t fat_alloc(void) {
    uint32_t total = bpb.total_sectors_16 ? bpb.total_sectors_16 : bpb.total_sectors_32;
    uint16_t max_c = (uint16_t)((total - data_start) / bpb.sectors_per_cluster + 2);
    for (uint16_t c = 2; c < max_c; c++) {
        if (fat_get(c) == 0x0000) { fat_set(c, 0xFFFF); return c; }
    }
    return 0;
}

// Имена

static void to83(const char* fn, char* out) {
    memset(out,' ',11);
    int i=0,j=0;
    while(fn[i]&&fn[i]!='.'&&j<8) out[j++]=toupper((unsigned char)fn[i++]);
    while(fn[i]&&fn[i]!='.') i++;
    if(fn[i]=='.'){i++;j=8; while(fn[i]&&j<11) out[j++]=toupper((unsigned char)fn[i++]);}
}

static void from83(const char* in, char* out) {
    int j=0;
    for(int i=0;i<8&&in[i]!=' ';i++) out[j++]=in[i];
    if(in[8]!=' '){ out[j++]='.'; for(int i=8;i<11&&in[i]!=' ';i++) out[j++]=in[i]; }
    out[j]='\0';
}

// Инициализация

int fat16_init(uint32_t ps) {
    (void)ps;
    if (rd(0,&bpb)!=0) return -1;
    if (bpb.bytes_per_sector!=512) return -1;
    fat_start        = bpb.reserved_sectors;
    root_dir_sectors = ((bpb.root_entries*32)+511)/512;
    root_dir_start   = fat_start + bpb.num_fats*bpb.fat_size_16;
    data_start       = root_dir_start + root_dir_sectors;
    return 0;
}

// Поиск файла по корневой директории

int fat16_find_file(const char* filename, fat16_dir_entry_t* entry) {
    // Есть ли слэш?
    const char* sl = filename;
    while(*sl && *sl!='/') sl++;

    if(*sl=='/') {
        char d83[11]; memset(d83,' ',11);
        int dlen=(int)(sl-filename); if(dlen>8)dlen=8;
        for(int i=0;i<dlen;i++) d83[i]=toupper((unsigned char)filename[i]);

        fat16_dir_entry_t de;
        int found=0;
        for(uint32_t s=0;s<root_dir_sectors&&!found;s++){
            if(rd(root_dir_start+s,sec_buf)!=0) return -1;
            fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
            for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
                if(E[i].name[0]==0x00) goto done_root_dir;
                if((uint8_t)E[i].name[0]==0xE5||!(E[i].attributes&0x10)) continue;
                if(memcmp(E[i].name,d83,8)==0){de=E[i];found=1;break;}
            }
        }
done_root_dir:
        if(!found) return -1;

        char f83[11]; to83(sl+1,f83);
        uint16_t clus=de.cluster_low;
        while(clus<0xFFF8){
            uint32_t lba=clus_lba(clus);
            for(uint8_t s=0;s<bpb.sectors_per_cluster;s++){
                if(rd(lba+s,sec_buf)!=0) return -1;
                fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
                for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
                    if(E[i].name[0]==0x00) return -1;
                    if((uint8_t)E[i].name[0]==0xE5||E[i].attributes==0x0F||E[i].attributes&0x08) continue;
                    if(memcmp(E[i].name,f83,11)==0){*entry=E[i];return 0;}
                }
            }
            clus=fat_get(clus);
        }
        return -1;
    }

    // Корневая директория
    char n83[11]; to83(filename,n83);
    for(uint32_t s=0;s<root_dir_sectors;s++){
        if(rd(root_dir_start+s,sec_buf)!=0) return -1;
        fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
        for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
            if(E[i].name[0]==0x00) return -1;
            if((uint8_t)E[i].name[0]==0xE5||E[i].attributes==0x0F||E[i].attributes&0x08) continue;
            if(memcmp(E[i].name,n83,11)==0){*entry=E[i];return 0;}
        }
    }
    return -1;
}

// Чтение

int fat16_read_file(const char* filename, void* buffer, uint32_t max_size) {
    fat16_dir_entry_t e;
    if(fat16_find_file(filename,&e)!=0) return -1;
    uint8_t* buf=(uint8_t*)buffer;
    uint32_t rd_total=0, fsz=e.file_size; if(fsz>max_size)fsz=max_size;
    uint16_t clus=e.cluster_low;
    while(clus<0xFFF8&&rd_total<fsz){
        uint32_t lba=clus_lba(clus);
        for(uint8_t s=0;s<bpb.sectors_per_cluster;s++){
            if(rd(lba+s,sec_buf)!=0) return -1;
            uint32_t tc=SECTOR_SIZE; if(rd_total+tc>fsz)tc=fsz-rd_total;
            memcpy(buf+rd_total,sec_buf,tc); rd_total+=tc;
            if(rd_total>=fsz) return (int)rd_total;
        }
        clus=fat_get(clus);
    }
    return (int)rd_total;
}

// Запись / создание файла (поддерживает путь DIR/FILE)

int fat16_write_file(const char* filename, const void* data, uint32_t size) {

    const char* sl = filename;
    while(*sl && *sl != '/') sl++;

    // Флаги где искать свободную/существующую запись
    int   in_subdir  = (*sl == '/');

    uint32_t dir_lba_base = 0;
    uint32_t dir_sectors  = 0;
    uint16_t dir_clus     = 0;   // 0 = корень

    if (in_subdir) {
        // Поиск поддиректории
        char d83[11]; memset(d83, ' ', 11);
        int dlen = (int)(sl - filename);
        if (dlen > 8) dlen = 8;
        for (int i = 0; i < dlen; i++)
            d83[i] = toupper((unsigned char)filename[i]);

        fat16_dir_entry_t de; int found = 0;
        for (uint32_t s = 0; s < root_dir_sectors && !found; s++) {
            if (rd(root_dir_start + s, sec_buf) != 0) return -1;
            fat16_dir_entry_t* E = (fat16_dir_entry_t*)sec_buf;
            for (uint32_t i = 0; i < SECTOR_SIZE/32; i++) {
                if (E[i].name[0] == 0x00) goto subdir_not_found;
                if ((uint8_t)E[i].name[0] == 0xE5) continue;
                if (!(E[i].attributes & 0x10)) continue;
                if (memcmp(E[i].name, d83, 8) == 0) { de = E[i]; found = 1; break; }
            }
        }
subdir_not_found:
        if (!found) return -1;

        dir_clus     = de.cluster_low;
        filename     = sl + 1;   // имя файла без пути
    }

    char n83[11]; to83(filename, n83);

    // Ищем существующую или свободную запись в нужной директории
    uint32_t esec = 0, eidx = 0; int efound = 0;
    fat16_dir_entry_t old; memset(&old, 0, sizeof(old));

    if (!in_subdir) {
        // Корневая директория
        for (uint32_t s = 0; s < root_dir_sectors && !efound; s++) {
            if (rd(root_dir_start + s, sec_buf) != 0) return -1;
            fat16_dir_entry_t* E = (fat16_dir_entry_t*)sec_buf;
            for (uint32_t i = 0; i < SECTOR_SIZE/32; i++) {
                uint8_t f = (uint8_t)E[i].name[0];
                if (!efound && (f == 0x00 || f == 0xE5)) { esec = s; eidx = i; efound = 1; break; }
                if (memcmp(E[i].name, n83, 11) == 0) { esec = s; eidx = i; old = E[i]; efound = 2; break; }
            }
        }
    } else {
        // Поддиректория: поиск по кластерам
        uint16_t clus = dir_clus;
        uint32_t abs_clus_idx = 0;
        (void)dir_lba_base; (void)dir_sectors;
        while (clus < 0xFFF8 && !efound) {
            uint32_t lba = clus_lba(clus);
            for (uint8_t s = 0; s < bpb.sectors_per_cluster && !efound; s++) {
                if (rd(lba + s, sec_buf) != 0) return -1;
                fat16_dir_entry_t* E = (fat16_dir_entry_t*)sec_buf;
                for (uint32_t i = 0; i < SECTOR_SIZE/32; i++) {
                    uint8_t f = (uint8_t)E[i].name[0];
                    if (!efound && (f == 0x00 || f == 0xE5)) {
                        esec = lba + s; eidx = i; efound = 1; break;
                    }
                    if (memcmp(E[i].name, n83, 11) == 0) {
                        esec = lba + s; eidx = i; old = E[i]; efound = 2; break;
                    }
                }
            }
            abs_clus_idx++;
            clus = fat_get(clus);
        }
    }
    if (!efound) return -1;
    // освобождение старых кластеров
    if (efound == 2 && old.cluster_low) {
        uint16_t c = old.cluster_low;
        while (c < 0xFFF8) { uint16_t nx = fat_get(c); fat_set(c, 0); c = nx; }
    }

    // Выделение новых кластеров и запись на них
    uint32_t csz = (uint32_t)bpb.sectors_per_cluster * SECTOR_SIZE;
    uint32_t cn  = (size + csz - 1) / csz; if (!cn) cn = 1;
    uint16_t first_c = 0, prev_c = 0;
    const uint8_t* src = (const uint8_t*)data;
    uint32_t wrt = 0;
    for (uint32_t ci = 0; ci < cn; ci++) {
        uint16_t c = fat_alloc(); if (!c) { fat_flush(); return -1; }
        if (prev_c) fat_set(prev_c, c); else first_c = c;
        fat_set(c, 0xFFFF); prev_c = c;
        uint32_t lba = clus_lba(c);
        for (uint8_t s = 0; s < bpb.sectors_per_cluster; s++) {
            memset(sec_buf2, 0, SECTOR_SIZE);
            uint32_t tw = (wrt < size) ? size - wrt : 0;
            if (tw > SECTOR_SIZE) tw = SECTOR_SIZE;
            if (tw) memcpy(sec_buf2, src + wrt, tw);
            wrt += tw;
            if (wr(lba + s, sec_buf2) != 0) { fat_flush(); return -1; }
        }
    }
    fat_flush();

    // Запись или обновление записи директории
    uint32_t dir_sector_lba = in_subdir ? esec : (root_dir_start + esec);
    if (rd(dir_sector_lba, sec_buf) != 0) return -1;
    fat16_dir_entry_t* E = (fat16_dir_entry_t*)sec_buf;
    memset(&E[eidx], 0, sizeof(fat16_dir_entry_t));
    memcpy(E[eidx].name, n83, 11);
    E[eidx].attributes  = 0x20;
    E[eidx].cluster_low = first_c;
    E[eidx].file_size   = size;
    if (wr(dir_sector_lba, sec_buf) != 0) return -1;
    return (int)size;
}

// Создание директории

int fat16_mkdir(const char* dirname) {
    char n83[11]; memset(n83,' ',11);
    for(int i=0;dirname[i]&&i<8;i++) n83[i]=toupper((unsigned char)dirname[i]);

    // Проверка существования
    for(uint32_t s=0;s<root_dir_sectors;s++){
        if(rd(root_dir_start+s,sec_buf)!=0) return -1;
        fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
        for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
            if(E[i].name[0]==0x00) goto mkdir_scan_done;
            if((E[i].attributes&0x10)&&memcmp(E[i].name,n83,8)==0) return 0;
        }
    }
mkdir_scan_done:;
    uint16_t c=fat_alloc(); if(!c) return -1;
    uint32_t lba=clus_lba(c);
    memset(sec_buf2,0,SECTOR_SIZE);
    for(uint8_t s=0;s<bpb.sectors_per_cluster;s++) wr(lba+s,sec_buf2);
    fat_flush();

    for(uint32_t s=0;s<root_dir_sectors;s++){
        if(rd(root_dir_start+s,sec_buf)!=0) return -1;
        fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
        for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
            uint8_t f=(uint8_t)E[i].name[0];
            if(f==0x00||f==0xE5){
                memset(&E[i],0,sizeof(fat16_dir_entry_t));
                memcpy(E[i].name,n83,11);
                E[i].attributes=0x10; E[i].cluster_low=c;
                wr(root_dir_start+s,sec_buf); return 0;
            }
        }
    }
    return -1;
}

// Список директории

int fat16_list_dir(const char* path) {
    int count=0;
    vga_set_color(VGA_COLOR_YELLOW,VGA_COLOR_BLACK);
    vga_puts("\n  Directory of /");
    if(path&&path[0]&&!(path[0]=='/'&&path[1]=='\0')){vga_puts(path);}
    vga_puts("\n\n");
    vga_set_color(VGA_COLOR_WHITE,VGA_COLOR_BLACK);

    // Корень
    if(!path||!path[0]||(path[0]=='/'&&!path[1])){
        for(uint32_t s=0;s<root_dir_sectors;s++){
            if(rd(root_dir_start+s,sec_buf)!=0) return -1;
            fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
            for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
                if(E[i].name[0]==0x00) goto root_ls_done;
                if((uint8_t)E[i].name[0]==0xE5||E[i].attributes==0x0F||E[i].attributes&0x08) continue;
                char fn[13]; from83(E[i].name,fn);
                if(E[i].attributes&0x10){
                    vga_set_color(VGA_COLOR_LIGHT_CYAN,VGA_COLOR_BLACK);
                    vga_puts("  ");vga_puts(fn);
                    for(int p=strlen(fn);p<14;p++)vga_putchar(' ');
                    vga_puts("<DIR>\n");
                } else {
                    int fl=strlen(fn);
                    if(fl>=4&&strcmp(&fn[fl-4],".ZXE")==0)
                        vga_set_color(VGA_COLOR_LIGHT_GREEN,VGA_COLOR_BLACK);
                    else
                        vga_set_color(VGA_COLOR_WHITE,VGA_COLOR_BLACK);
                    vga_puts("  ");vga_puts(fn);
                    for(int p=strlen(fn);p<14;p++)vga_putchar(' ');
                    vga_set_color(VGA_COLOR_DARK_GREY,VGA_COLOR_BLACK);
                    vga_put_dec(E[i].file_size);vga_puts(" bytes\n");
                }
                count++;
            }
        }
root_ls_done:;
        vga_set_color(VGA_COLOR_WHITE,VGA_COLOR_BLACK);
        vga_puts("\n  "); vga_put_dec(count); vga_puts(" item(s)\n\n");
        return count;
    }

    // Поддиректория
    const char* dn=(path[0]=='/')?path+1:path;
    char d83[11]; memset(d83,' ',11);
    int di=0; while(dn[di]&&dn[di]!='/'&&di<8){d83[di]=toupper((unsigned char)dn[di]);di++;}

    fat16_dir_entry_t de; int df=0;
    for(uint32_t s=0;s<root_dir_sectors&&!df;s++){
        if(rd(root_dir_start+s,sec_buf)!=0) return -1;
        fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
        for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
            if(E[i].name[0]==0x00) break;
            if(!(E[i].attributes&0x10)) continue;
            if(memcmp(E[i].name,d83,8)==0){de=E[i];df=1;break;}
        }
    }
    if(!df){vga_puts("  Not found\n\n");return -1;}

    uint16_t clus=de.cluster_low;
    while(clus<0xFFF8){
        uint32_t lba=clus_lba(clus);
        for(uint8_t s=0;s<bpb.sectors_per_cluster;s++){
            if(rd(lba+s,sec_buf)!=0) return -1;
            fat16_dir_entry_t* E=(fat16_dir_entry_t*)sec_buf;
            for(uint32_t i=0;i<SECTOR_SIZE/32;i++){
                if(E[i].name[0]==0x00) goto subls_done;
                if((uint8_t)E[i].name[0]==0xE5||E[i].attributes==0x0F||E[i].attributes&0x08) continue;
                char fn[13]; from83(E[i].name,fn);
                int fl=strlen(fn);
                if(fl>=4&&strcmp(&fn[fl-4],".ZXE")==0)
                    vga_set_color(VGA_COLOR_LIGHT_GREEN,VGA_COLOR_BLACK);
                else
                    vga_set_color(VGA_COLOR_WHITE,VGA_COLOR_BLACK);
                vga_puts("  ");vga_puts(fn);
                for(int p=strlen(fn);p<14;p++)vga_putchar(' ');
                vga_set_color(VGA_COLOR_DARK_GREY,VGA_COLOR_BLACK);
                vga_put_dec(E[i].file_size);vga_puts(" bytes\n");
                count++;
            }
        }
        clus=fat_get(clus);
    }
subls_done:
    vga_set_color(VGA_COLOR_WHITE,VGA_COLOR_BLACK);
    vga_puts("\n  "); vga_put_dec(count); vga_puts(" item(s)\n\n");
    return count;
}

int fat16_list_root(void) { return fat16_list_dir(""); }

uint32_t fat16_file_size(const char* fn) {
    fat16_dir_entry_t e; if(fat16_find_file(fn,&e)!=0)return 0; return e.file_size;
}
int fat16_file_exists(const char* fn) {
    fat16_dir_entry_t e; return fat16_find_file(fn,&e)==0;
}