This is a server and client implementation of a recreation of an old game.

# Building
**Ubuntu**
*Requirements*
* Packages - libsdl2-mixer-dev

*Building*
* cargo build

**Windows**
*Requirements*
* TODO

**OSX**
*Requirements*
* TODO

# Server
See server/examplesettings.ini for a sample configuration file. It should be modified as needed and saved as server-settings.ini.

# Powerpc mac emulation
1. qemu-img create -f qcow2 mac.qcow2 25G
2. qemu-system-ppc -L pc-bios -boot d -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -drive file=~/Downloads/10.5\ Leopard.dmg,format=dmg,media=cdrom
3. qemu-system-ppc -L pc-bios -boot c -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk