# Theory of operation

**Stylus** acts as a webserver with special endpoints, and a status monitoring
tool.

The status monitoring portion is based around scripts, written in any shell
scripting language you like. Each script is run regularly at an interval, and if
the script returns `0` that is considered "up" for a given service. If the
service times out, or returns a non-zero error this is considered a soft
("yellow") or hard ("red") failure.

The special endpoints available on the webserver are:

- `/style.css`: A dynamically generated CSS file based on the current
- `/status.json`: A JSON representation of the current state

The `style.css` endpoint may be linked by a HTML or SVG file served from the
`static` directory that is configured. If desired, the HTML page can dynamically
refresh the CSS periodically using Javascript. See the included example for a
sample of how this might work.

If you need more flexibility than CSS can provide, you can use the `status.json`
endpoint to get the current status of the various services and dynamically
update the HTML DOM, SVG images, a React.js application or something more
complex.
