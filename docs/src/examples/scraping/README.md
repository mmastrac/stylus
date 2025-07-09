# Web/HTML/API Scraping

Some devices can be tested using a simple cURL script, while others require more complex HTML parsing and API interaction.

## Simple cURL Check

```bash
curl --silent --max-time 2 <url>
```

## HTML Scraping with pup and jq

In other cases you may want to scrape HTML. The `pup` tool is included in the docker image to make this easier. You can use the `json{}` filter to pass a pre-processed HTML DOM tree to `jq` for further processing.

This example scrapes the power state from a Web Power Switch 7:

```bash
#!/bin/bash
set -euf -o pipefail

function fetch() {
    curl --silent --max-time 2 --basic -u <credentials> <url> \
        | pup -c "table table tr[bgcolor=#F4F4F4] json{}" \
        | jq "[.[] | [.children | .. | .text? | select(. != null)] | { \"name\": .[1], \"state\": (.[2]==\"ON\") }]"
}

n=0
until [ "$n" -ge 10 ]
do
   HTML=`fetch` && break
   n=$((n+1)) 
done

echo $HTML | \
    jq -r -e ". | to_entries | .[] | \"@@STYLUS@@ group.power-\" + (.key + 1 | tostring) + \".status.status=\" + if .value.state then \"\\\"green\\\"\" else \"\\\"blank\\\"\" end"
```

## When to Use Scraping

HTML/API scraping is ideal for:
- Web-based device monitoring
- API endpoint health checks
- Web application monitoring
- Devices with web interfaces
- Custom dashboard monitoring

For simpler connectivity tests, consider [ping monitoring](../ping/). For server monitoring, consider [SSH monitoring](../ssh/). 