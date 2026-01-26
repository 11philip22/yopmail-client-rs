# yopmail-client

[![Crates.io](https://img.shields.io/crates/v/yopmail-client.svg)](https://crates.io/crates/yopmail-client)
[![Documentation](https://docs.rs/yopmail-client/badge.svg)](https://docs.rs/yopmail-client)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![YOPmail](https://yopmail.com/logo.png)

Unofficial async Rust client for [YOPmail](https://yopmail.com). It mirrors the web UI flow (cookies + `yp` tokens) to list inboxes, fetch message bodies (text + HTML), download attachments, send mails, and work with RSS feeds. Ships with a small CLI example.

## Install
```bash
cargo add yopmail-client
```

## Quickstart (library)
```rust
use yopmail_client::{generate_random_mailbox, YopmailClient};

#[tokio::main]
async fn main() -> Result<(), yopmail_client::Error> {
    let mailbox = "mytempbox";
    let mut client = YopmailClient::new(mailbox)?;

    // List first page
    let messages = client.check_inbox_page(1).await?;

    // Fetch plain text
    let body = client.get_message_by_id(&messages[0].id).await?;

    // Fetch full content (html/raw/attachments)
    let content = client.get_message_by_id_full(&messages[0].id).await?;
    for att in &content.attachments {
        println!(
            "attachment: {} -> {}",
            att.name.clone().unwrap_or_default(),
            att.url
        );
    }

    // Download an attachment
    let bytes = client.download_attachment(&content.attachments[0]).await?;

    // Generate a random mailbox name
    let random_box = generate_random_mailbox(12);
    println!("{random_box}@yopmail.com");

    Ok(())
}
```

Customize the client with the builder when you need a proxy, timeout, or base URL override:
```rust
use std::time::Duration;
use yopmail_client::YopmailClient;

let mut client = YopmailClient::builder("mytempbox")
    .proxy_url("http://127.0.0.1:8080")
    .timeout(Duration::from_secs(20))
    .build()?;
```

## CLI (examples/cli.rs)
```bash
cargo run --example cli -- --mailbox mytempbox list --details
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id>
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id> --html
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id> --attachments
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id> --download-attachments downloads/
cargo run --example cli -- random --len 10
```

Commands: `list`, `fetch`, `send`, `rss-url`, `rss-data`, `info`, `random`. Use `--proxy` to tunnel through a proxy.

## Features
- Inbox: list with paging (`YopmailClient::check_inbox_page`, `list_messages`).
- Fetch: text/HTML/raw plus attachment discovery (`get_message_by_id_full`, `fetch_message_full`).
- Attachments: download via `download_attachment`.
- Send: post to another `@yopmail.com` address.
- RSS: get feed URL and parse items.
- Helpers: inbox counts/summaries, random mailbox generator.

## Available domains
```
0cd.cn, 1nom.org, 1xp.fr, 15963.fr.nf, a.kwtest.io, abo-free.fr.nf, ac-malin.fr.nf, actarus.infos.st, adresse.biz.st, adresse.infos.st, afw.fr.nf, altrans.fr.nf, alves.fr.nf, alphax.fr.nf, alyxgod.rf.gd, antispam.fr.nf, antispam.rf.gd, assurmail.net, autre.fr.nf, bahoo.biz.st, bboys.fr.nf, bibi.biz.st, bin-ich.com, binich.com, blip.ovh, c-eric.fr.nf, cabiste.fr.nf, calendro.fr.nf, calima.asso.st, carioca.biz.st, carnesa.biz.st, cc.these.cc, certexx.fr.nf, cloudsign.in, cobal.infos.st, contact.biz.st, contact.infos.st, cookie007.fr.nf, cool.fr.nf, courriel.fr.nf, cpc.cx, cubox.biz.st, dann.mywire.org, dede.infos.st, degap.fr.nf, desfrenes.fr.nf, dis.hopto.org, dlvr.us.to, dmts.fr.nf, donemail.my.id, dreamgreen.fr.nf, dripzgaming.com, druzik.pp.ua, ealea.fr.nf, elmail.4pu.com, emaildark.fr.nf, emocan.name.tr, enpa.rf.gd, eooo.mooo.com, faybetsy.com, fhpfhp.fr.nf, fiallaspares.com, flobo.fr.nf, flaimenet.ir, freemail.biz.st, frostmail.fr.nf, galaxim.fr.nf, get.route64.de, get.vpn64.de, gimuemoa.fr.nf, gland.xxl.st, ggamess.42web.io, ggmail.biz.st, gmail.gob.re, gladogmi.fr.nf, haben-wir.com, habenwir.com, himail.infos.st, hunnur.com, iamfrank.rf.gd, iuse.ydns.eu, imap.fr.nf, internaut.us.to, isep.fr.nf, ist-hier.com, iya.fr.nf, jmail.fr.nf, jetable.fr.nf, jetable.org, jinva.fr.nf, kyuusei.fr.nf, lacraffe.fr.nf, le.monchu.fr, lerch.ovh, likeageek.fr.nf, m.tartinemoi.com, ma.ezua.com, ma.zyns.com, ma1l.duckdns.org, mabal.fr.nf, machen-wir.com, mail.berwie.com, mail.hsmw.net, mail.i-dork.com, mail.kakator.com, mail.tbr.fr.nf, mail.xstyled.net, mail.yabes.ovh, mailadresi.tk, mailbox.biz.st, mailsafe.fr.nf, mai.25u.com, mcdomaine.fr.nf, mc-fly.be, mickaben.biz.st, mickaben.fr.nf, mickaben.xxl.st, miloras.fr.nf, miistermail.fr, mondial.asso.st, moncourrier.fr.nf, monemail.fr.nf, monmail.fr.nf, mynes.com, mymail.infos.st, mymailbox.xxl.st, mymaildo.kro.kr, myself.fr.nf, nikora.biz.st, nikora.fr.nf, nidokela.biz.st, noreply.fr, nospam.fr.nf, noyp.fr.nf, omicron.token.ro, pamil.fr.nf, pepamail.com, pixelgagnant.net, pitimail.xxl.st, pliz.fr.nf, pochtac.ru, pokemons1.fr.nf, pooo.ooguy.com, popol.fr.nf, poubelle-du.net, poubelle.fr.nf, q0.us.to, rapidefr.fr.nf, randol.infos.st, readmail.biz.st, redi.fr.nf, rygel.infos.st, sdj.fr.nf, sendos.fr.nf, sendos.infos.st, sirttest.us.to, six25.biz, sind-hier.com, sind-wir.com, sindhier.com, sindwir.com, skynet.infos.st, spam.aleh.de, spam.quillet.eu, super.lgbt, tagara.infos.st, terre.infos.st, test-infos.fr.nf, test.inclick.net, tivo.camdvr.org, tmp.x-lab.net, toolbox.ovh, torrent411.fr.nf, totococo.fr.nf, tshirtsavvy.com, tweet.fr.nf, upc.infos.st, ves.ink, vip.ep77.com, vitahicks.com, vigilantkeep.net, webclub.infos.st, webstore.fr.nf, whatagarbage.com, wishy.fr, wir-sind.com, woofidog.fr.nf, wxcv.fr.nf, yaloo.fr.nf, yahooz.xxl.st, y.dldweb.info, ym.cypi.fr, ym.digi-value.fr, yop.iotf.net, yop.kd2.org, yop.kyriog.fr, yop.mabox.eu, yop.mc-fly.be, yop.moolee.net, yop.smeux.com, yop.too.li, yop.uuii.in, yopmail.ca, yopmail.co.ke, yopmail.co.nz, yopmail.co.pl, yopmail.co.uk, yopmail.com, yopmail.com.au, yopmail.de, yopmail.es, yopmail.fr, yopmail.gr, yopmail.hk, yopmail.id, yopmail.in, yopmail.ir, yopmail.it, yopmail.jp, yopmail.kr, yopmail.lt, yopmail.net, yopmail.net.cn, yopmail.net.fr, yopmail.org, yopmail.pl, yopmail.pp.ua, yopmail.pt, yopmail.se, yopmail.sg, yopmail.tw, yopmail.uz, yopmail.za, yopmail.kro.kr, yopmail.ozm.fr, yopmail.pp, yopmail.to, yopmail.too.li, yopmail.uuii.in, yopmail.xl.cx, yopmail.ydns.eu, yopmail.mabox.eu, yopmail.moolee.net, yopmail.smeux.com, yopmail.net3 … yopmail.net200 (all numeric suffix variants), yopmail.pp.ua, ypmpail.sehier.fr, ypmpail.sehier.fr, ypmpail.sehier.fr, ypmpail.sehier.fr, ypmpail.sehier.fr, yopmail.pp.ua, yopmail.net.in, yopmail.mx, yopmail.my, yopmail.me, yopmail.ch, yopmail.be, yopmail.at, yopmail.hu, yopmail.cz, yopmail.sk, yopmail.si, yopmail.hr, yopmail.ba, yopmail.rs, yopmail.bg, yopmail.ro, yopmail.ua, yopmail.by, yopmail.lv, yopmail.ee, yopmail.ge, yopmail.am, yopmail.kz, yopmail.tj, yopmail.az, yopmail.iq, yopmail.il, yopmail.sa, yopmail.qa, yopmail.ae, yopmail.pk, yopmail.lk, yopmail.bd, yopmail.th, yopmail.ph, yopmail.vn, yopmail.la, yopmail.kh, yopmail.mm, yopmail.cn, yopmail.co.jp, yopmail.ng, yopmail.ke, yopmail.tz, yopmail.rw, yopmail.ug, yopmail.mg, yopmail.ma, yopmail.dz, yopmail.tn, yopmail.ly, yopmail.eg
```

## Notes
- All network helpers are async; run them inside an async runtime such as `tokio`.
- Alternative domains: pass any domain from the list above via `--mailbox user@domain` (CLI) or `YopmailClient::new("user@domain", ...)`.
- Network is live scraping of YOPmail; availability and captcha/rate limits are outside this client’s control.
- Send only to the allowed YOPmail domains listed above.
- Attachment parsing uses the webmail DOM (links with class `pj` or `downmail` URLs).

## Acknowledgements
This project is a Rust port of the [Python yopmail-client](https://pypi.org/project/yopmail-client/1.2.3/). This port does not require a license key 😉

## License
This project is licensed under the MIT License - see the [license](license) file for details.
