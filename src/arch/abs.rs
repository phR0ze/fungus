// Arch Linux ABS
// https://wiki.archlinux.org/index.php/Arch_Build_System
// https://wiki.archlinux.org/index.php/Arch_Build_System#Retrieve_PKGBUILD_source_using_Git
//
// # Clone the svntogit repo for a specific package using https://github.com/archlinux/asp
// # asp replaces the abs tool, offering more up to date sources (via the svntogit repositories)
// # using a sparse checkout to cache at ${XDG_CACHE_HOME:-$HOME/.cache}/asp.
// $ asp checkout <pkgname>
