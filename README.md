# CloudFlare Dynamic DNS 

This Rust program allows you to use the free CloudFlare DNS service as a dynamic DNS provider.
It provides functions to get your external IPv4 and IPv6 addresses, get the ID of a DNS zone, and create or update a DNS record in that zone.

This is a strongly inspired by [hotio's excellent CloudFlareDDNS tool](https://hotio.dev/containers/cloudflareddns/). Initially, I wrote this to practice writing Rust code.

## Usage
To use this program, you will need a CloudFlare account and an API key.
You can find instructions for how to generate an API key in the [CloudFlare documentation](https://support.cloudflare.com/hc/en-us/articles/200167836-Where-do-I-find-my-Cloudflare-API-key-).

Once you have an API key, you can set the environment variables to configure the program.

It will your external IP address(es), get the ID of the specified DNS zones, and create or update a DNS record in those zones.

## Environment

* `CLOUDFLAREDDNS_USER` and `CLOUDFLAREDDNS_APIKEY`: The username/API key for your CloudFlare account. These are used to authenticate with the CloudFlare API.
* `CLOUDFLAREDDNS_RECORDTYPES`: A semicolon-separated list of record types to create/update. This can include `A`, `AAAA`, or both.
* `CLOUDFLAREDDNS_HOSTS`: A semicolon-separated list of hostnames to update. This should be the name of the DNS record that you want to create/update in each zone.
* `CLOUDFLAREDDNS_ZONES`: A semicolon-separated list of zone names to update. This should be the name of the DNS zone that contains the record that you want to update.
* Optional: `CLOUDFLAREDDNS_IPV4_API_ENDPOINT`/`CLOUDFLAREDDNS_IPV6_API_ENDPOINT`: The API endpoint to use to obtain the current external IPv4/IPv6 address, respectively. By default, CloudFlareDDNS uses the `https://api.ipify.org` and `https://api6.ipify.org` endpoints.

Note that `CLOUDFLAREDDNS_HOSTS` and `CLOUDFLAREDDNS_ZONES` are parsed as parallel arrays. In other words, they must contain a matching number of elements,
and each element's position in the variable should match the position of the corresponding element in the other variable.

## License
This program is licensed under the GPLv3 license. See LICENSE for more information.
