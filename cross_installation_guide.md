# Cross installation guide
To setup cross compilation:
- `cargo install cross`
- install and enable docker (for ex. `sudo dnf install docker`)
	- reboot or enable docker `sudo systemctl start docker`
	- add user to group `sudo usermod -aG docker $USER`
		- relogin or update groups via `newgrp docker`
