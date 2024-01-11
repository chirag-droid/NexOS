# GNU Grub

NexOS uses GNU Grub to chainload a EFI executable `/EFI/boot.efi` which does the actual loading of the kernel.

In theory, you could just directly load `/EFI/boot.efi` without using Grub but it just simplifies the whole process.

Grub provides following features out of the box.

- Basic scripting support
- GUI (better bootsplash support, custom colors, custom themes, ...)
- Memory management
- Cleaner design
- Better portability
- Internationalization
- Rescue mode

## Theme

I will change the theme moving forward just wanted to see what is possible.

I am using [Minegrub theme](https://github.com/Lxtharia/minegrub-theme) licenced under MIT Licence.

See `grub/themes/minegrub/LICENCE` for details.
