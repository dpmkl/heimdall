# heimdall - a https reverse proxy #
Heimdall is a https reverse proxy to act as a single gateway for multiple http sites, requiring only a single https setup. It utlizies hyper for http/2 handling and is heavily based on async/await. 

It strips the hop-by-hop headers, adds or extends the 'x-forwarded-for' header with the client ip and returns an unmodified http response from the backend. 

As heimdall utilizes native_tls a DER-formated PKCS #12 archive is required. 
```bash
openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem -certfile chain_certs.pem
```

## Usage ## 
Write a default config file to <CONFIG_FILE>
```bash
heimdall default <CONFIG_FILE>
```

Run heimdall from a config file <CONFIG_FILE>
```bash
heimdall run <CONFIG_FILE>
```

## Notes ##
Due to async/await being activley developed on the nighly channel it is not recommend to use in production yet. 