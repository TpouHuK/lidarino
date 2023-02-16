# Cross installation guide
To setup cross compilation:
- `cargo install cross` 

As of Feb 16 2023 on Fedora, it requires docker installation from the official website, because of introduction of build_tools or smthing.
- install and enable docker (for ex. `sudo dnf install docker`)
	- reboot or enable docker `sudo systemctl start docker`
	- add user to group `sudo usermod -aG docker $USER`
		- relogin or update groups via `newgrp docker`
