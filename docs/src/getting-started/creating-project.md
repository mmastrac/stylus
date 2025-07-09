# Creating a **Stylus** Project

You can create a new **Stylus** project by running the `stylus init` command. This
will create a new directory with a default configuration and a `monitor.d`
directory with a single monitor test.

```bash session
$ stylus init ~/stylus
Initializing directory: "~/stylus"...
Done!

Run `stylus "~/stylus"` to start the server

$ tree ~/stylus/
  ├── config.yaml
  ├── monitor.d
  │   └── monitor
  │       ├── config.yaml
  │       └── test.sh
  └── static
      └── README.md
```

Once you've created the project, you can run the `stylus run` command to start
the server.

```bash
stylus run ~/stylus
```

If you open your web browser to `http://localhost:8000`, you should see a
very basic default page with a green status. You'll also find a link to the
status JSON and style CSS endpoints, as well as the per-monitor log output:

<blockquote style="background-color: #f3faff !important; color: black !important;">
<h1 style="color: black !important;">Stylus</h1>
<p>Updated at 2025-07-08T22:46:07.257Z</p>
<table style="border: 1px solid #ccc; border-collapse: collapse;">
    <tbody><tr>
        <th>Monitor</th>
        <th>Status</th>
        <th>Exit</th>
        <th>Log</th>
    </tr>
    <tr data-monitor-id="monitor" style="background-color: green;">
        <td>monitor</td>
        <td>green</td>
        <td>Success (0)</td>
        <td><a style="color: blue; text-decoration: underline;">Log</a></td>
    </tr>
</tbody></table>
<ul>
    <li><a style="color: blue; text-decoration: underline;">Status JSON</a></li>
    <li><a style="color: blue; text-decoration: underline;">Style CSS</a></li>
</ul>
</blockquote>

By default, **Stylus** will render a basic summary page for all of your monitors,
which allows you to work on your monitors before you've created any pages.

