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
**mysql database**
Setup a mysql database at the place specified in the configuration file.

# Powerpc mac emulation (work in progress)
1. `qemu-img create -f qcow2 mac.qcow2 50G`
2. `qemu-system-ppc -L pc-bios -boot d -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -drive file=~/Downloads/10.5\ Leopard.dmg,format=dmg,media=cdrom`
3. Create a file in /etc/systemd/network/90-tap0.netdev
```
[NetDev]
Name=tap0
Kind=tap

[Tap]
Group=<your desired group>
```
4. Create a file in /etc/systemd/network/90-tap0.network
```
[Match]
Name=tap0

[Network]
Address=11.11.11.11/24
```
5. Create a file in /etc/systemd/network/90-br0.network
```
[Match]
Name=br0

[Network]
Address=11.11.11.10/24
```
5. Restart systemd-networkd with systemctl: `systemctl restart systemd-networkd` (enabling the systemd-networkd service may be also required with `systemctl enable`)
6. Run these commands
```
sudo brctl addbr br0
sudo brctl addif br0 tap0
sudo ip link set br0 up
sudo ip link set tap0 up
```
7. Add this to your .ssh/config
```
Host lineage
	HostName 11.11.11.12
	User lineage
	KexAlgorithms diffie-hellman-group-exchange-sha1,diffie-hellman-group1-sha1
	HostKeyAlgorithms ssh-rsa,ssh-dss
```
8. `sudo qemu-system-ppc -L pc-bios -boot c -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -device sungem,netdev=network01 -netdev tap,ifname=tap0,id=network01`
9. On the osx host, download and install xcode from `http://home.uglyoldbob.com/downloads/xcode313_2736_developerdvd.dmg`
10. On the osx host, download and install lineage client from `http://home.uglyoldbob.com/downloads/Lineage_Tikal_Antharas_Update.dmg`
11. On the osx host, configure networking for static ip address at 11.11.11.12 with subnet of 255.255.255.0
12. On the osx host, enable remote login in sharing (found in system preferences, sharing).
13. On the osx host, change sleep options to never on both putting computer and display to sleep in system preferences, energy saver