# log-sender

This application collects logs written as json object lines to `*.jsonl` files.

The json must be stringified and written on a single line.
It must have 2 properties: "k" and "t".

- `k` - is the log "kind". Holochain uses "start" and "fetchedOps".
- `t` - is the microsecond (!!`Date.now() * 1000`!!) timestamp as a string type.

Example: `{"k":"start","t":"1758571617392359"}`.

The line can contain any additional properties as desired.

It is expected that whatever process is writing the logs will also rotate them.
The log-sender will never delete any log files.

Build `log-sender` locally with `make all`!

## running a local log-collector to aid in developing log-sender

```
# check out log-collector
git clone git@github.com:unytco/log-collector.git

# go into dir
cd log-collector

# get deps
npm install

# run the local test server
npx wrangler dev

# in a different console, initialize the d1 database
npx wrangler d1 execute log-collector-db --local --file=./schema.sql
```
