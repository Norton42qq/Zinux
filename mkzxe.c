#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

typedef struct {
    uint32_t magic;
    uint8_t  version;
    uint8_t  flags;
    uint16_t header_size;
    uint32_t code_size;
    uint32_t data_size;
    uint32_t bss_size;
    uint32_t entry_offset;
    char     name[16];
    char     reserved[8];
} __attribute__((packed)) zxe_header_t;

int main(int argc, char* argv[]) {
    if (argc < 3) {
        printf("Usage: %s <input.bin> <output.zxe> [name]\n", argv[0]);
        return 1;
    }
    
    const char* input_file = argv[1];
    const char* output_file = argv[2];
    const char* name = (argc >= 4) ? argv[3] : "program";
    
    FILE* fin = fopen(input_file, "rb");
    if (!fin) {
        printf("Error: cannot open %s\n", input_file);
        return 1;
    }
    
    fseek(fin, 0, SEEK_END);
    long code_size = ftell(fin);
    fseek(fin, 0, SEEK_SET);
    
    unsigned char* code = malloc(code_size);
    fread(code, 1, code_size, fin);
    fclose(fin);
    
    zxe_header_t header;
    memset(&header, 0, sizeof(header));
    
    header.magic = 0x01455A58;
    header.version = 1;
    header.flags = 0;
    header.header_size = sizeof(zxe_header_t);
    header.code_size = code_size;
    header.data_size = 0;
    header.bss_size = 0;
    header.entry_offset = 0;
    strncpy(header.name, name, 15);
    
    FILE* fout = fopen(output_file, "wb");
    if (!fout) {
        printf("Error: cannot create %s\n", output_file);
        free(code);
        return 1;
    }
    
    fwrite(&header, 1, sizeof(header), fout);
    fwrite(code, 1, code_size, fout);
    fclose(fout);
    
    free(code);
    
    printf("Created %s: header=%lu bytes, code=%ld bytes, total=%lu bytes\n",
           output_file, sizeof(header), code_size, sizeof(header) + code_size);
    
    return 0;
}