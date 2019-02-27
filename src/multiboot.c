#include "multiboot.h"

boot_info_t read_boot_tags(const void *ptr)
{
	const multiboot_tag_t *tag = ptr + 8;

	boot_info_t data;
	bzero(&data, sizeof(data));

	while(tag->type != MULTIBOOT_TAG_TYPE_END)
	{
		switch(tag->type)
		{
			case MULTIBOOT_TAG_TYPE_CMDLINE:
			{
				data.cmdline = ((multiboot_tag_string_t *) tag)->string;
				break;
			}

			case MULTIBOOT_TAG_TYPE_BOOT_LOADER_NAME:
			{
				data.loader_name = ((multiboot_tag_string_t *) tag)->string;
				break;
			}

			case MULTIBOOT_TAG_TYPE_MODULE:
			{
				// TODO
				break;
			}

			case MULTIBOOT_TAG_TYPE_BASIC_MEMINFO:
			{
				data.mem_lower
					= ((multiboot_tag_basic_meminfo_t *) tag)->mem_lower;
				data.mem_upper
					= ((multiboot_tag_basic_meminfo_t *) tag)->mem_upper;
				break;
			}

			case MULTIBOOT_TAG_TYPE_BOOTDEV:
			{
				// TODO
				break;
			}

			// TODO

			default: {}
		}

		tag = (multiboot_tag_t *) ((uint8_t *) tag + ((tag->size + 7) & ~7));
	}

	return data;
}