# Mapping the Internet
## Running
1. Setup a PostgreSQL database, accessible by all services which need it (Pokedex, Pidgeotto, Cubone).
2. Create [database accounts](https://www.postgresql.org/docs/current/sql-createrole.html) for the units. Make sure to grant read/write access to the appropriate tables. The list of the tables with which the service communicates is listed in the service's README, together with the required permissions (select/insert/update/delete).
3. Run a Pokedex unit to authenticate and register units. You need to generate an RSA key pair and save it in the PEM format so that Pokedex can sign the generated JWT.
4. Base minimum is running one Pidgeotto unit, one Pidgey unit and one Diglett unit. These need to communicate together, so either run them on the same server or make them publically available. When exposing the units to the public, it is recommended to keep the Diglett API behind an auth gateway (setting api.auth to true, which is also the default).

## Services
Here is the full list of services and what they do:

- Pokedex - The service list and authentication provider
- Pidgeotto - Manages the scanning process, utilising its pidgey instances
- Pidgey - Gathers information about an address
- Diglett - Translation layer for plaintext data to JSON
- Cubone - API used to access the collected data
- Bulbasaur - Web application for viewing the data

### Diagram
![Services diagram](./services-diagram.png)

## Address
We want to know these things about a specific address:
- What is its allocation state (Unknown, Reserved, Unallocated, Allocated)
- Whether it is routed
- Whether it is online
- Which RIR this address originally falls under (/8 block segments)
- Which RIR does this address belong to
- Which ASN does this address belong to

Here is the address record structure from the database
```yml
- id: string
- allocation_state_id: bool
- top_rir_id: int
- rir_id: int
- asn_id: int
- country_id: int
- routed: bool
- online: bool
```

### Allocation states
An IPv4 address can be in one of the following allocation states:
- `unknown` - self explanatory
- `reserved` - the address is reserved
- `unassigned` - the address wasn't yet assigned to a register or the register has not yet given it away
- `assigned` - the address was assigned

## TODO
- [ ] (Diglett) implement automatic downloads of asn prefixes file with cron
- [ ] (Pidgeotto) Implement job queue and rework the scanning to progresivelly scan and add to queue
- [ ] (Pidgeotto) Implement stale address records via updated_at timestamps
- [ ] (Pidgey) Return the 500 error when Diglett responds with a 500 error
- [ ] (Pidgeotto) Implement a timeout when waiting for a job response from Pidgey
- [ ] (Diglett) Finish the transition to Settings
- [ ] (Pokedex) Update example config and .gitignore
- [ ] (Pidgeotto) Update example config and .gitignore
- [x] (Pidgey) Update example config and .gitignore
- [x] (Pidgey) Test Pidgeotto connect procedure
