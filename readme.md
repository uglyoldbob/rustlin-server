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
* `sudo apt install mysql-server`
* `sudo mysql`
* mysql> `CREATE USER 'lin'@'localhost' IDENTIFIED BY 'password';`
* mysql> `CREATE DATABASE l1jdb;`
* mysql> `GRANT ALL PRIVILEGES ON l1jdb.* TO 'lin'@'localhost';`
* mysql> `FLUSH PRIVILEGES;`
* mysql> `exit`
* `sudo mysql -p l1jdb < ./l1jdb_m7.sql` (this command will not produce output)


# Powerpc mac emulation (work in progress)
1. `qemu-img create -f qcow2 mac.qcow2 50G`
2. `qemu-system-ppc -L pc-bios -boot d -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -drive file=~/Downloads/10.5\ Leopard.dmg,format=dmg,media=cdrom`
3. On the osx host, download and install xcode from `http://home.uglyoldbob.com/downloads/xcode313_2736_developerdvd.dmg`
* NOTE: Https website will probably not work at all due to the web browsers not having newer https encryption methods required
4. On the osx host, download and install lineage client from `http://home.uglyoldbob.com/downloads/Lineage_Tikal_Antharas_Update.dmg`
5. Create a file in /etc/systemd/network/90-tap0.netdev
```
[NetDev]
Name=tap0
Kind=tap

[Tap]
Group=<your desired group>
```
6. Create a file in /etc/systemd/network/90-tap0.network
```
[Match]
Name=tap0

[Network]
Address=11.11.11.11/24
```
7. Create a file in /etc/systemd/network/90-br0.network
```
[Match]
Name=br0

[Network]
Address=11.11.11.10/24
```
8. Restart systemd-networkd with systemctl: `systemctl restart systemd-networkd` (enabling the systemd-networkd service may be also required with `systemctl enable`)
9. Add this to your .ssh/config
```
Host lineage
	HostName 11.11.11.12
	User lineage
	KexAlgorithms diffie-hellman-group-exchange-sha1,diffie-hellman-group1-sha1
	HostKeyAlgorithms ssh-rsa,ssh-dss
```
10. Add yourself to the netdev group. 
```
sudo usermod -a -G netdev <your username>
newgrp netdev
```
11. `qemu-system-ppc -L pc-bios -boot c -M mac99,via=pmu -m 512 -drive file=./mac.qcow2,format=qcow2,media=disk -device sungem,netdev=network01 -netdev tap,ifname=tap0,id=network01`
12. On the osx host, configure networking for static ip address at 11.11.11.12 with subnet of 255.255.255.0
13. On the osx host, enable remote login in sharing (found in system preferences, sharing).
14. On the osx host, change sleep options to never on both putting computer and display to sleep in system preferences, energy saver
15. On the osx host, change the Desktop and Screen saver options, change the screen saver to never start. (It uses much more cpu time when running).