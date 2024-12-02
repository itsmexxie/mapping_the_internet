# Mapping the Internet - Diglett
A service that provides a nice HTTP abstraction over services that provide address information via text files.

## Service list
The services which we are currently using to get information are as follows:
- ftp.arin.net (allocation states, RIRs, countries)
- IANA number resources (RIRs, reserved blocks)
- thyme.apnic.net (RIRs, ASNs)

## TODO
- [ ] implement automatic downloads of asn prefixes file with cron
- [x] change rir algorithm to account for recovered addresses (via. [delegation stats](https://ftp.arin.net/pub/stats/lacnic/delegated-lacnic-latest))
- [x] implement provider for arin stats files
- [ ] implement provider for rdap responses (maybe inflight requests only? no static files?)
- [x] implement provider for iana number files
