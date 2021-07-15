
<p  align="center">

<a  href="https://dscvit.com">

<img  src="https://user-images.githubusercontent.com/30529572/92081025-fabe6f00-edb1-11ea-9169-4a8a61a5dd45.png"  alt="DSC VIT"/>

</a>

<h2  align="center"> Securum Exire </h2>

<h4  align="center"> A extensive solution to prevent security credential leaks, at response phase of a request-response cycle. <h4>

</p>

  

---

[![Join Us](https://img.shields.io/badge/Join%20Us-Developer%20Student%20Clubs-red)](https://dsc.community.dev/vellore-institute-of-technology/)

[![Discord Chat](https://img.shields.io/discord/760928671698649098.svg)](https://discord.gg/498KVdSKWR)
## Linked Projects
- [SecurumExireSignalServer](https://github.com/mayankkumar2/SecurumExireSignalServer) - Signal Server Implementation
- [exirectl](https://github.com/mayankkumar2/exirectl) - Securum Exire Controller Implementation
- [SecurumExireBot](https://github.com/mayankkumar2/SecurumExireBot) - Securum Exire Bot Implementation
- [traefik-plugin-securum-exire](https://github.com/mayankkumar2/traefik-plugin-securum-exire) - Securum Exire Telegram Bot Implementation
## Features

- [X] Telegram Bot to notify of leaks.
- [X] Fast and Effective Algorithm to find credentials in a particular response.
- [X] System synchronisation for credentials available on the system.
- [X] Signal server to communicate with the telegram bot server.
- [X] JWT key exchange
- [X] Service discovery for signal server and leaks scanner server
- [X] Environment variable scraping cli
  


## Architecture Diagram
![image](https://user-images.githubusercontent.com/24864829/124398308-af0e8600-dd32-11eb-891c-3a1176b4b822.png)
  

## Dependencies

- Rust
- Golang
- Traefik
- Redis
- Telegram Bot


## Steps to install and run

  
- Extract the contents of [installer.tar.gz](https://github.com/mayankkumar2/securum-exire/releases/download/v0.1.0/installer.gz.tar)<br>
command: 
```

tar -xvf installer.tar.gz

```

- Change the permissions for install.sh<br>

command:

```

chmod +x install.sh

```

  

- Get the BOT UID and BOT SECRET from telegram bot [Securum Exire bot](http://t.me/SecurumExireBot)<br>

  

- Expose a port to public internet OR install ngrok (preferred for new users)

  
- Get public webhook ready.

    - METHOD 1 (if you have a exposed port of your operating node):<br>

        the webhook address will be: 
  ```
  http://<YOUR_PUBLIC_IP>:10000
  ```

    - METHOD 2 (ngrok method): <br>
        command:<br>
  ```bash
  ngrok http 10000
  ```
  the webhook is the NGROK URL provided to you by ngrok cli

-  Run the script<br>

```

./install.sh

```

  

- Go to secumum exire install location <br>

```

cd $HOME/securum_exire

```

  

- Start the service

```

./startup.sh

``` 

- Check the logs with

```

tail -f <LOCATION_PROMPTED_BY_STARTUP_SCRIPT>

```

  

- Service is up and running.

  

- Use the [traefik-plugin-securum-exire](https://github.com/mayankkumar2/traefik-plugin-securum-exire) with traefik to utilize the service.
# Troubleshooting

## Why is the secrets scan not working?

### Securum exire scan tool
-  If the contents of ```$HOME/.securum_exire/credentials.json``` are empty you can run the following command to populate the file with required credentials.
```bash
./securum_exire -o <PATH_TO_SCAN> -p ~/.securum_exire 
Credentials written at path [/Users/mayankk/go/src/credentials.json]
```

## Contributors

  

<table>

<tr  align="center">

<td>

Mayank Kumar

<p  align="center">

<img  src = "https://dscvit.com/images/techteam/mayank.jpg"  width="150"  height="150"  alt="Mayank Kumar">

</p>

<p  align="center">

<a  href = "https://github.com/mayankkumar2">

<img  src = "http://www.iconninja.com/files/241/825/211/round-collaboration-social-github-code-circle-network-icon.svg"  width="36"  height = "36"  alt="GitHub"/>

</a>

<a  href="https://www.linkedin.com/in/mayankk2">

<img  src = "http://www.iconninja.com/files/863/607/751/network-linkedin-social-connection-circular-circle-media-icon.svg"  width="36"  height="36"  alt="LinkedIn"/>

</a>

</p>

</td>

</tr>

</table>

  
<center>
<p  align="center">
Made with :heart: by <a  href="https://dscvit.com">DSC VIT</a>
</p>
  </center>
