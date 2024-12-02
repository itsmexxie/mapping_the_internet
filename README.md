# Mapping the Internet
## Address
We want to know these things about a specific address:
- What is its allocation state (Unknown, Reserved, Unallocated, Allocated)
- Whether it is routed
- Whether it is online
- Which RIR this address originally falls under (/8 block segments)
- Which RIR does this address belong to
- Which ASN does this address belong to
- What ports are open on this server
- What services are available on the server

Here is the address record structure from the database
```yml
- id: string
- allocation_state_id: bool
- routed: bool
- online: bool
- top_rir_id: int
- rir_id: int
- asn_id: int
- country_id: int
```

### States
An IPv4 address can be in one of the following states:
- `reserved` - the address is reserved
- `unassigned` - the address wasn't yet assigned to a register
- `assigned unrouted` - the address was assigned, but no routers know how to get there
- `assigned routed offline` - the address was assigned and is routed (and the route is publicly known via protocols like BGP)
- `assigned routed online` - the address was assigned, is routed and is connected to a live computer

