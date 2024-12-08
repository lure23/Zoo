# `ble-web-app`

A web application for interacting with a local BLE embedded product.

`../ble/`: source code for the BLE device


## Requirements

- `node.js` 
- `npm`

If you wish a ready package, see [`mp > web`](https://github.com/akauppi/mp/tree/main/web) (GitHub).

## Steps

```
$ npm install
```

That installs the dependencies listed in `package.json`.

```
$ npm run dev
[...]
Forced re-optimization of dependencies

  VITE v5.4.11  ready in 27092 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
  ➜  press h + enter to show help
```

If you are using Multipass for virtualization, that `localhost` is within your VM, not the host. There are two ways to access it:

**A. Use your VM's IP**

```
[host]$ mp info {vm-name}
[...]
IPv4:           192.168.64.149
[...]
```

With that IP, open [`http://192.168.64.149:5173`](http://192.168.64.149:5173).

This approach is easier, but you need to remember to use the VM's IP. Also, the IP is bound to change at times.


**B. Port forward 5173**

This approach makes the port `5173` usable - as `localhost:5173` - from your host. However, it requires you to:

- run `sudo` on the host
- leave a window open for the duration of the port forward

Follow the instructions [within the `mp` repo](https://github.com/akauppi/mp/tree/main/web#using-installing-a-cli):

```
$ sudo ssh -i /var/root/Library/Application\ Support/multipassd/ssh-keys/id_rsa -L 5173:localhost:5173 ubuntu@192.168.64.149

# keep the terminal open
```

Open [`localhost:5173`](http://localhost:5173).


## Deployment (optional)

The application is made with deployment to Cloudflare Pages in mind, but you can easily change the SvelteKit adapter to your choosing.

### Access rights

Follow the steps [here](https://github.com/akauppi/mp/tree/main/web%2Bcf#b-login-with-custom-api-tokens).

- Create an API token for this application

	- Set the access rights mentioned on the above linked page<br />
	+ `Users` > `Memberships` > `Read`

- export it as `CLOUDFLARE_API_TOKEN` env.var.

```
$ wrangler whoami

 ⛅️ wrangler 3.87.0 (update available 3.88.0)
-------------------------------------------------------

Getting User settings...
ℹ️  The API Token is read from the CLOUDFLARE_API_TOKEN in your environment.
👋 You are logged in with an API Token, associated with the email demo@outstanding.earth.
┌───────────────────┬─────────────────────────────┐
│ Account Name      │ Account ID                  │
├───────────────────┼─────────────────────────────┤
│ Outstanding Earth │ ...8<8<8< snipped 8<8<8<... │
└───────────────────┴─────────────────────────────┘
```

### Deploy manually

```
$ npm run deploy

> ble-web-app@0.0.1 deploy
> npm run build && wrangler pages deploy


> ble-web-app@0.0.1 build
> vite build

vite v5.4.11 building SSR bundle for production...
✓ 144 modules transformed.
vite v5.4.11 building for production...
✓ 121 modules transformed.
.svelte-kit/output/client/_app/version.json                                    0.03 kB │ gzip:  0.04 kB
.svelte-kit/output/client/.vite/manifest.json                                  2.79 kB │ gzip:  0.52 kB
.svelte-kit/output/client/_app/immutable/chunks/legacy.B5bMJODb.js             0.04 kB │ gzip:  0.06 kB
[...]
✓ built in 4.11s
.svelte-kit/output/server/.vite/manifest.json                  1.65 kB
[...]
✓ built in 32.79s

Run npm run preview to preview your production build locally.

> Using @sveltejs/adapter-cloudflare
  ✔ done
? The project you specified does not exist: "ble-web-app". Would you like to create it? › - Use arrow-keys. Return to submit.
❯   Create a new project
[...answer some prompts...]

✨ Successfully created the 'ble-web-app' project.
✨ Success! Uploaded 15 files (2.87 sec)

✨ Uploading _headers
✨ Compiled Worker successfully
✨ Uploading Worker bundle
✨ Uploading _routes.json
🌎 Deploying...
✨ Deployment complete! Take a peek over at https://b66fc1e5.ble-web-app.pages.dev
```

Great!!!

The URL you get is for the particular deployment.

If you don't need to share these things, nothing prevents you from just copy-pasting that and using it as such.

>macOS hint:
>
>Cmd-double-click on such a URL. :)


## What this means

![](.images/ble-human-cf.png)

We can now browse to a website

..that provides a UI

..that can find and control a BLE device in our proximity!


### ..for security

For demo purposes, you can leave the web app unprotected (if there are no secrets in its content itself). In order for anyone to steer the device, they need to <u>both know the URL and be in the proximity of such a device</u>.

If there should be access restrictions, the same authentication mechanisms that you'd use for any web page can be applied (password, social login, corporate login).

## ..for security (take 2!)

But what about the BLE security? Forget about the web page - if the BLE interface is unprotected, people can just use a suitable monitoring software and steer your device.

True.

For pairing, you can require certain numbers to be entered. You can likely hide the device. But this is something the author is only approaching. Browsing the web, you'll likely get answers (after all, BLE is already 14 years old!) - and ideally, you'd write something about it *right here*. :)
