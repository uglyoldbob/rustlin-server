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

# Powerpc mac emulation
1. `qemu-img create -f qcow2 mac.qcow2 25G`
2. `qemu-system-ppc -L pc-bios -boot d -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -drive file=~/Downloads/10.5\ Leopard.dmg,format=dmg,media=cdrom`
3. Create a file in /etc/systemd/network/90-tap0.netdev
```
[NetDev]
Name=tap0
Kind=tap

[Tap]
Group=<your desired group>
```
4. Restart systemd-networkd with systemctl: `systemctl restart systemd-networkd`
5. Run these commands
```
sudo brctl addbr br0
sudo brctl addif br0 tap0
sudo ip link set br0 up
sudo ip link set tap0 up
sudo ip route add default via 11.11.11.1 dev br0
sudo ip addr add 11.11.11.11/24 dev tap0
```
6. Add this to your .ssh/config
```
Host lineage
	HostName 11.11.11.12
	User lineage
	KexAlgorithms diffie-hellman-group-exchange-sha1,diffie-hellman-group1-sha1
	HostKeyAlgorithms ssh-rsa,ssh-dss
```
6. `sudo qemu-system-ppc -L pc-bios -boot c -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -device sungem,netdev=network01 -netdev tap,ifname=tap0,id=network01`