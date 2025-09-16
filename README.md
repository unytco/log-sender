# log-sender

## quick start

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
