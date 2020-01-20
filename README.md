# heimdall - a https reverse proxy #
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Heimdall is a https reverse proxy to act as a single gateway for multiple http sites, requiring only a single https setup. It utlizies hyper for http/2 handling  based on async/await. 

It strips the hop-by-hop headers, adds or extends the 'x-forwarded-for' header with the client ip and returns an unmodified http response from the backend. 

This project is still in its infancy, so beware. 

## Usage ## 
### TLS ### 
Heimdall is intended to run in LetsEncrypt ACME scenarios an therefore requires the certificate chain and private key file to be PEM formated.

For testing purposes a self signed certificate can be created with
```bash
openssl req -x509 -newkey rsa:4096 -keyout privkey.pem -out fullchain.pem -days 365 -nodes
```

### Standalone binary ### 
1. Write a config file to <CONFIG_FILE> and adjust accordingly
```bash
heimdall default $CONFIG_FILE
```

2. Run heimdall from config file <CONFIG_FILE>
```bash
heimdall run $CONFIG_FILE
```

### RPM (systemd service) ### 
1. Build RPM if necessary 
```bash
./publish_rpm.sh 
export RPM="./target/release/rpmbuild/RPMS/<ARCH>/heimdall-X.Y.Z-N.arch.rpm"
```
2. Install with 'su' or 'sudo'
```bash
sudo yum install $RPM 
# or 
sudo dnf install $RPM 
```
3. Create a configuration file and adjust accordingly
```bash
sudo heimdall default /etc/heimdall.toml
# If you use a different path for the config file 
# make sure to edit the heimdall.service file accordingly
``` 
4. Start service 
```bash
sudo systemctl start heimdall 
```
