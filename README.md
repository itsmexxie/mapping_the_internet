# Mapping the Internet
## Address
We want to know these things about a specific address:
- Whether it is assigned or reserved
- Whether it is routed
- Whether it is online
- What services are available on the server
- Which RIR this address falls under
- Who owns this address
- Which ASN does this address fall under

```yml
- id: string # The actual IP address
- state_id: bool # Whether the IP address is assigned, unassigned or reserved
- routed: bool # Whether the IP address is routed
- online: bool # Whether the IP address is online
- rir_id: int
- asn_id: int
```

### States
An IPv4 address can be in one of the following states: 
- `reserved` - the address is reserved
- `unassigned` - the address wasn't yet assigned to a register
- `assigned unrouted` - the address was assigned, but no routers know how to get there
- `assigned routed offline` - the address was assigned and is routed (and the route is publicly known via protocols like BGP)
- `assigned routed online` - the address was assigned, is routed and is connected to a live computer

