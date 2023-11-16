<div align="center">
<h1>knewsnotifd</h1>
Kernel news RSS notifier daemon for Discord webhooks
<br/><br/>
<img src="https://i.imgur.com/8g66aPL.png" alt="Example RSS message"/>
</div>
<hr/>

knewsnotifd is a Discord webhook that can notify users in a server of new Linux kernel updates

## Installation

### systemd

1. Copy `knewsnotifd.service` to `/etc/systemd/system`

2. `chmod 644 /etc/systemd/system/knewsnotifd.service`

Give the newly added service the correct permissions

3. `systemctl daemon-reload`

Reload the systemd daemon to add the new service

4. `systemctl enable --now knewsnotifd.service`

Enable and run the new service

### Docker

1. `chmod +x setup_docker.sh`

Give the setup script execution permissions

2. Edit the Dockerfile and change `KNEWSNOTIFD_WEBHOOK_URL`'s definition to your webhook URL

3. `./setup_docker.sh`

Run the setup script

4. Add the newly built Docker container to a Docker compose file or run it indivually