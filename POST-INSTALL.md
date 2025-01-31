# Post-installation instructions

These instructions will vary depending on your operating system. I can only hope it works on Mac, but I have no way of testing it at this moment

## Linux (Ubuntu, maybe others)
By default [udev](https://en.wikipedia.org/wiki/Udev) denies access to devices by non-root users.
Meaning you can run luxafor-ui using `sudo luxafor-ui`, but I would advice you to add this udev rule


To give yourself access you must explicitly add the device through a udev rule

Run the included [udev-cfg](./udev-cfg.sh) script
```bash
sudo ./udev-cfg.sh
```


Or manually copy the included [udev rules file](./udev-rule/99-luxafor-ui.rules) into the rules folder and reload udev:

```bash
# Copy udev rule into rules directory
sudo cp ./udev-rule/99-luxafor-ui.rules /etc/udev/rules.d
# Reload udev
sudo udevadm control -R
```

After this, unplug and plug back in the Flag. You should now be able to control the busylight


## Windows
Nothing. It should just work.


## Mac
/shrug

