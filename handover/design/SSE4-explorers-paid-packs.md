# Paid packs on the Explorers site

Status: **accepted by Alex, 21 Nov 2026** ("Stripe Connect - yup", "Yes to recs"): every recommendation in section 13 is now a ruling. Proposed the same day. Alex ruled the same day that all of the charging is to be built now so it can be tried and tested, behind one master switch under admin that is off by default (section 1a). The build runs in the payment provider's test mode; no real money moves until the accountant's answers in section 12 are in and Alex gives his explicit yes to live keys. Nothing is built yet.

This is practical design, not legal or tax advice. Section 12 lists what to confirm with an accountant before any real money moves.

Figures marked **[read 21 Nov]** were read from the provider's or HMRC's own page that day. Figures marked **[from memory]** need checking before they are relied on.

## 1. The short answer

- Creators of original packs can set a price of a few pounds on the Explorers site. Buyers pay by card on a Stripe-hosted page. The site keeps 20% of what is left after tax and card fees, the creator gets 80%, paid monthly to their bank by Stripe.
- "A standard agent" is read here as **a standard payment provider that also pays the sellers**. The recommendation is **Stripe Connect**. It is the only mainstream option that both takes the payment and pays many independent sellers. The "we handle all the tax for you" providers (Paddle, Lemon Squeezy, Stripe's own Managed Payments) either forbid marketplaces or are closing.
- The catch is VAT. HMRC treats a site like this as the seller of every pack, so **your existing VAT-registered business is the seller on every receipt** and accounts for the VAT. UK sales go on your normal VAT return. EU sales need one extra registration (the non-Union One Stop Shop) before the first EU sale, because there is no small-seller threshold for a UK business selling downloads into the EU.
- SSE4 itself never takes money and gets no accounts. A purchase lives on the buyer's Explorers account. The pack is an ordinary file once downloaded: it works offline, in a self-hosted copy, for ever, with a signed receipt naming the buyer and no copy protection.
- Fan work based on someone else's setting can never carry a price (Alex's ruling, 21 Nov). The site refuses the price itself rather than relying on people to behave.
- Building can start invisibly. The first slice (section 11) is pure code with tests, no database change, no route and no screen.

## 1a. Alex's rulings, 21 Nov

- **Build all of the charging now**, to try and test. One **master switch under admin** turns the monetisation features on; it is off by default. In the site this is a row on the existing admin config page, changed by an admin only and written to the audit log. While it is off, no price, buy button, seller page or payment route exists for anyone.
- **The cut is shown openly and honestly.** The buyer sees the split on the pack's page and on the receipt before paying: for example "£3.00: 50p VAT, 26p card fees, £1.79 to the creator, 45p to Explorers for hosting". The seller sees the same sum live as they type a price. Nothing is taken that is not on that line.
- **Free is always free.** Nothing that is free today gains a price, a limit or a nag. Free and paid packs sit on the same shelves with the same features; a free pack is never ranked below a paid one for being free. The site's share exists to pay for hosting so that everyone else can host free work at no charge, and the site says so.
- **Creators can always host their own work elsewhere.** A pack is a file. SSE4 imports it from anywhere, sold or given away by any means, with no fee to the site; it is simply less integrated (no shelf, no one-click add). Selling on Explorers is never exclusive.
- **What can carry a price is any Explorers item**: a crafted game world (a map or system, the item types the site has today) as well as a pack. "Pack" in this document means any of them. This also means the charging can be built and tested on maps now, without waiting for pack shelves (TG15).
- **Fan work based on other people's settings cannot be monetised**, and a price is blocked on anything that is not the seller's own (section 7).
- **SSE4 itself never takes money. Alex takes the cut only and does not sell packs himself.**
- His existing business is VAT-registered and can be the seller (section 3).

## 2. What exists already and is reused

The Explorers site already has most of the skeleton. Nothing below is new:

| Need | What the site has |
|---|---|
| Buyer and seller identity | Explorers accounts with confirmed email (Supabase), roles user, moderator, admin, and a "trusted" mark |
| "Who is entitled to what, and why" | The entitlement ledger (`entitlements.ts`): grants with a source, an outside reference, an expiry and a revoke, written for Patreon. Purchases use the same shape |
| SSE4 proving who the user is without a password | Device pairing (`devicePairing.ts`): the app shows a short code, the user approves it in the browser |
| Files never served raw | Private R2, every download goes through a gate that checks the moderation ledger |
| Rights statement | The versioned attestation (`attestation.ts`) and the fan-work field (`fanWork.ts`, D-61) |
| Complaints | The report route, the takedown route and the audit log |
| A switch that is off | Config gates (`config.ts`) |

So the new parts are small: a price on a pack, a checkout, a purchase ledger, a seller's payout account, and the rules that refuse a price.

## 3. Provider, and who the seller is

### 3.1 Why the site is the seller whatever we choose

HMRC's guidance on digital services sold through a platform is blunt: the platform accounts for the VAT, not the creator, whenever the platform sets the terms, authorises the payment, or handles the download **[read 21 Nov]**. Explorers would do all three. The EU rule is the same in substance **[from memory]**. So there is no version of this where each creator is quietly their own little shop and the site is only a noticeboard, unless creators sell somewhere else entirely (option C).

That makes the legal shape a reseller one: the creator licenses the pack to your business, your business sells it to the buyer, and the creator's 80% is what your business pays them for it.

### 3.2 The options

| | A. Stripe Connect (recommended) | B. A merchant-of-record provider | C. Creators sell elsewhere |
|---|---|---|---|
| How it works | Stripe takes the card payment into your Stripe account and pays each creator's share to their bank | The provider is the legal seller, does all VAT everywhere, and pays you the net | Creator sells on itch.io or Gumroad, Explorers only checks a key |
| Pays many sellers | Yes, this is what it is for | No. Paddle's policy bans "digital marketplaces" **[read 21 Nov]**. Stripe Managed Payments lists Connect as unsupported and requires products sold "not through a platform or marketplace" **[read 21 Nov]**. Lemon Squeezy stopped taking new sellers on 16 Nov 2026 and closes 31 Mar 2027 (reported by a comparison site, not checked at source) | Not needed |
| VAT | Yours: UK return plus the EU One Stop Shop | Theirs | Theirs |
| Fee on a £3 UK sale | about 25p | about 35p to 55p | 60p and up |
| Your 20% | Automatic | You would pay creators by hand from your own bank | None |

Two smaller providers (Creem, Whop) advertise revenue splits or marketplace sub-accounts. Both claims come from marketing pages, neither is a company to hang other people's money on yet, and neither removes the question of who the seller is.

Because your business is already VAT-registered, option A costs you one extra registration rather than a new tax life, and it is the cheapest per sale. That is why it wins. If Stripe ever opens Managed Payments to marketplaces, the checkout can move to it with no change to anything else in this design.

### 3.3 VAT in practice under option A

- **UK buyers.** 20% VAT is inside the price, declared on your existing return. The £90,000 threshold does not help because you are already registered.
- **EU buyers.** See 3.4, written to be handed to the accountant.
- **Everyone else.** Most countries have a threshold far above anything this will earn at first (the US states, Australia, Canada) and a few do not. The accountant should say which, if any, to block at launch.
- **Stripe Tax** works out the right rate, shows it at checkout, and keeps the two pieces of location evidence HMRC asks for. It costs 0.5% of each sale in the countries where you are registered **[read 21 Nov]**. It does not file the returns.
- **Prices are shown with tax included**, as UK consumer law expects. A creator sets one sticker price in pounds; the tax inside it varies by buyer.

### 3.4 The EU registration, for the accountant

What is asked for: registration of the business for the **non-Union scheme of the EU VAT One Stop Shop (OSS)**. It is the successor to "VAT MOSS", and HMRC's guidance still uses the old name.

- **Why it is needed.** Since Brexit a UK business selling downloads (legally "electronically supplied services") to private consumers in the EU owes VAT in each buyer's own country, at that country's rate, from the first sale. The EUR 10,000 small-seller threshold is only for businesses established in the EU, so it does not apply. HMRC's guidance gives two choices: register for the non-Union scheme in one member state, or register for VAT separately in every member state sold into **[read 21 Nov]**.
- **What it is.** One registration, in one EU country of the business's choosing, that covers consumer sales of digital services in all 27. Ireland is the usual choice for a UK business because everything is in English; it is done online with Irish Revenue **[from memory]**.
- **What it involves afterwards.** One return each calendar quarter listing sales and VAT per country, and one payment in euros to that country's tax office, which passes the money on. A return is due even for a quarter with no EU sales. Records are kept for ten years. It is separate from the UK VAT return and does not change it **[from memory: deadlines and the ten years]**.
- **What it does not do.** It gives no way to reclaim EU VAT on costs, and it covers consumer sales only. That suits this case: the only thing sold into the EU is packs to private buyers.
- **Timing.** A registration normally starts from the next quarter, or from the date of the first sale if it is notified promptly afterwards **[from memory, confirm]**. So register before EU sales are switched on. The site has a separate setting, "sell to EU buyers", off until the registration exists, so the UK and the rest of the world can go first if the paperwork lags.
- **What the site supplies for the return.** Stripe Tax works out each sale's country and rate and keeps the location evidence. The site's purchase ledger gives a quarterly report: per country, net sales and VAT, in euros at the European Central Bank rate for the last day of the quarter **[from memory: the rate rule]**.
- **What to ask.** Which member state; what they charge to file the quarterly return; whether the business's sales to EU consumers through any other channel must go on the same return (they must be, if there are any); and the points in section 12.

## 4. The money on a small sale

Stripe's UK card fee is 1.5% + 20p for a standard UK card, 2.5% + 20p for an EEA card and 3.15% + 20p elsewhere, plus 2% if currency is converted **[read 21 Nov]**. The fixed 20p is what hurts on small prices.

Worked for a UK buyer on a standard card, with Stripe Tax, tax and fees taken off first and then split 80 to 20:

| Sticker price | VAT | Card and tax fee | Left to split | Creator (80%) | Site (20%) |
|---|---|---|---|---|---|
| £1 | 17p | 22p | 61p | 49p | 12p |
| £2 | 33p | 24p | £1.43 | £1.14 | 29p |
| £3 | 50p | 26p | £2.24 | £1.79 | 45p |
| £5 | 83p | 30p | £3.87 | £3.10 | 77p |
| £8 | £1.33 | 36p | £6.31 | £5.05 | £1.26 |

Three things follow.

- **"20% off the top" needs pinning down** (question 4). If the 20% were taken from the price before fees and the site then paid the fees, the site would keep about 24p of a £3 sale and lose money on a £1 one. The recommendation is the table above: tax and card fees come off first because neither of you keeps them, then 80 to 20.
- **A minimum price of £2**, with £3, £5 and £8 offered as the usual steps. Below £2 the fixed fee eats a quarter of the sale.
- **Paying creators has its own cost.** Stripe charges the platform about £2 for each creator paid in a month plus 0.25% + 10p per payout **[from memory in pounds; the page read gave the same in another currency]**. So pay out once a month, and only when a creator is owed £25 or more. The recommendation is that this comes out of the site's 20%, since it is small against a £25 payout and "you get 80%" stays true.

Disputes are the one nasty number: a chargeback costs £20 **[read 21 Nov]** on a £3 sale. The defence is to make a refund easier than a chargeback (section 6) and to let Stripe's own fraud checks run.

## 5. Sellers: joining, the declaration, getting paid

- **Who can sell.** At first, by invitation: creators who already have good free work on the site and carry the "trusted" mark. Opened wider later. This keeps the first months free of strangers arriving only to sell.
- **Joining.** The creator presses "Set up payouts" and is taken to Stripe's own onboarding (a Connect Express account). Stripe checks their identity and holds their bank details. The site stores only Stripe's account id and whether it is ready. A creator in a country Stripe cannot pay cannot sell.
- **The seller declaration.** One more versioned statement, kept beside the existing attestation and stored with each answer the same way: the pack and everything in it is my own or licensed to me for sale; it is not based on anyone else's setting; it contains nothing under a non-commercial licence; I grant the site the right to sell it; I am responsible for my own income tax; refunds and takedowns come out of my balance.
- **Getting paid.** The creator's share of each sale sits in the site's own ledger for 30 days (the refund window, section 6), then counts toward the monthly payout. The creator sees a plain statement: each sale, the tax, the fee, their share, what is pending and what has been paid.
- **Updates.** A new version of a paid pack is free to everyone who bought any earlier version. A sequel is a new pack.

## 6. Refunds, disputes, and packs that go wrong

- **The law.** A buyer of a download has 14 days to cancel, but loses that right once the download starts if they ticked a box agreeing to that **[from memory]**. The checkout has that box.
- **The policy, kinder than the law.** A refund on request within 14 days if the pack has not been downloaded, or at any time in the first 30 days if it is broken or not as described. Asked for through the site, decided by an admin, one click. It is cheaper than one chargeback.
- **What a refund does.** The purchase is marked revoked in the ledger (the same revoke the entitlement ledger has), the creator's pending share is taken back, and the card fee is lost by the site, since Stripe does not return it **[read 21 Nov]**. A copy already downloaded keeps working; there is nothing to switch off and we do not pretend otherwise.
- **Chargebacks.** The site answers them and absorbs the £20. The sale is taken back from the creator's balance. A creator whose packs draw repeated disputes loses the right to sell.
- **A pack that should never have been sold** (someone else's setting, lifted content). Delisted at once through the existing takedown route, every buyer refunded, the money taken from the seller's held balance first and their future sales after. The 30-day hold exists largely for this.
- **A seller who disappears.** Purchased packs stay downloadable by their buyers. The listing is closed to new sales.

## 7. What may carry a price

Decided by Alex, 21 Nov: fan-made work based on other people's settings cannot be monetised, and a price is blocked on anything using content that is not the seller's. The site enforces it in four places, so it does not depend on anyone reading the rules.

1. **The fan-work field refuses a price.** The site already asks which setting a work is based on. If that field is filled, or a moderator marks the item as fan work, the price box is not offered and a price sent anyway is refused. This covers tips, paid tiers and bundles too: a paid pack cannot include or require a fan pack as part of what is sold.
2. **Licences refuse a price.** Every asset in an upload already has a recorded licence. A price is refused while any asset is non-commercial (CC BY-NC and its variants), "used with permission" without a commercial statement, or uncredited. The refusal names the files.
3. **The seller declaration** (section 5) covers what a machine cannot check.
4. **A person looks before the first sale.** The first version of every paid pack goes through the moderation queue before it can be bought. Later versions go live at once unless reported.

Reports get one new reason, "sold without the right to sell it". A moderator marking a paid pack as fan work delists and refunds it in one action (section 6).

Fan packs keep their own shelf, apart from anything with a price, as the tags design already says (section 5.7 there).

## 8. Gaia data, and the Apache licence

**Apache-2.0: nothing changes.** The licence allows commercial use, the app stays free, and a pack is data that belongs to its creator, not a derivative of the app. Sellers may build on the stock packs shipped in SSE4, since those are Apache too.

**Copyright and the company: no change needed.** Alex offered to tie the copyright to his company if this needed it. It does not. Who owns the code's copyright and who operates the shop are separate things. The NOTICE file can go on crediting Alex personally. The company only has to be named as the operator and seller in the site's terms, the seller agreement, the receipts and the footer. Creators keep the copyright in their packs and grant the company a non-exclusive licence to sell them. The one thing worth a line from the accountant is whether the company should have a written licence from Alex to run his software and brand, which is a formality.

**Gaia: taking money makes the existing ruling matter, and adds two rules.** Gaia data is CC BY-NC 3.0 IGO, non-commercial. It already lives only in the optional real-sky pack.

- **The real-sky pack is never sold, never bundled into anything sold, and is served from SSE4's own storage, not from the shop.** Free, with its licence shown, as planned.
- **A paid pack may need the real-sky pack but may not contain it.** A creator can sell "my campaign set among the real stars within 50 light years". What they sell is their own work: the bases, factions, routes, histories and invented planets. The star data itself is fetched free by the buyer from the real-sky pack, exactly as any user gets it.

**How the stars are kept out.** SSE4 already marks every measured value as catalogued and pins the catalogue and version in the run manifest. Two small rules use that:

- When SSE4 exports a campaign or pack, a catalogued object is written as **a reference** (catalogue, version, source id) plus the creator's own additions, not as a copy of the measured values. This is right in any case: it is smaller and it keeps catalogue updates clean. It needs each catalogue to carry a licence flag in the register that package X5 already plans.
- The Explorers upload check refuses a price on any pack that still contains values from a non-commercial catalogue, naming them, the same way it refuses a non-commercial picture.

Stars that are also in free catalogues (Hipparcos, the bright-star lists) can be copied from those. The few catalogues whose terms are still unclear (CNS5, AT-HYG) are treated as non-commercial until the citations thread confirms them at source.

## 9. How a purchase reaches SSE4

SSE4 stays as it is: no accounts, no payment code, nothing to do with money inside the app.

1. **Buying happens on the website.** The Explorers shelf inside SSE4 shows a price and a "Buy on Explorers" button, which opens the pack's page in the browser. The buyer signs in to Explorers and pays on Stripe's hosted page. Card details never touch the site.
2. **The purchase is a row in the ledger** against the buyer's Explorers account.
3. **Getting the pack.** Either "Add to my campaign" in SSE4, which uses the existing device pairing to prove who is asking and fetches the pack, or "Download" on the website, which gives a file to import by hand.
4. **What arrives** is the pack plus a small signed receipt: pack id, content hash, buyer's handle, order number, date. SSE4 shows "Licensed to (handle)" on the pack in Workshop. That is all it does with it.

**No copy protection.** SSE4 is open source and a pack is readable data, so any lock would be removed in an afternoon and would only ever punish honest buyers when they are offline. The receipt naming the buyer is the same light touch tabletop PDF shops use, and it is enough for a few-pound product.

**Offline and self-hosted use.** Once downloaded the pack works with no connection, in any copy of SSE4 including a self-hosted one, with no check back to the site, ever. Losing the site would not break anyone's campaign.

**Players at the table.** The GM's purchase covers the table. Players never receive the pack: the GM's machine works out the results and sends only data, which is already how SSE4 works.

**Sharing what you made with it.** A campaign export names the packs it needs by id and hash and does not carry them. Someone who opens it without a paid pack sees everything the GM adopted and a line saying what the missing pack would add and where to get it. Publishing such a map on Explorers is allowed. Re-uploading the pack itself is not, and the content-hash ledger catches an exact copy.

## 10. What the site must hold

| Record | Contents | Kept |
|---|---|---|
| Listing | price, currency, seller, pack id and version hashes | while listed |
| Seller | Stripe account id, ready or not, declaration version and date, country | while selling, then 6 years |
| Purchase | buyer account, pack and version, sticker price, tax amount and country, fee, the split, Stripe payment id, state (paid, refunded, disputed) | 6 years for UK VAT, 10 for EU sales |
| Payout ledger | each creator's pending, available and paid amounts, each movement with its reason | 6 years |
| Receipt signing key | one private key, a Worker secret, never in the repository | rotated yearly |

Not held, by design: card numbers, bank details, identity documents (all Stripe's), and anything about what a buyer does with a pack.

The privacy page and terms need a buyer section and a seller agreement, and must say that purchase records survive account deletion because tax law requires it. The site's footer needs your business's name, address and VAT number, since it is now a seller.

## 11. Build packages

Owner for all Explorers packages: the thread "Integrate Explorers site with SSE4". Alex's ruling of 21 Nov is the go-ahead to build PP1 to PP7 now, in Stripe's test mode. All of it sits behind the master switch, the admin config row `paid_packs`, off by default, and every route answers "not found" while it is off. With the switch on and test keys in place, a yellow "TEST MODE, no real money" band shows on every money screen.

Merges on that repository deploy live and migrations are run by hand, so two guards hold throughout: every migration only adds new tables (no existing table or stored map is altered), and Alex is shown each migration before it is run. Live keys and PP8 wait for his explicit yes.

**What only Alex can do**

1. Open a Stripe account for the business and turn on Connect (test mode needs no verification and is free).
2. Put four values into the Worker's secrets in Cloudflare himself: the Stripe test secret key, the webhook signing secret, the publishable key, and the receipt signing key. These are never pasted into a chat, committed, or logged; the code reads them by name only and refuses to start the money routes without them.
3. Run each migration in Supabase, or say that the build thread may.
4. Flip the master switch, on the admin config page.
5. Later, for real money: Stripe's business verification, live keys, the accountant's answers, the One Stop Shop registration, and the terms.

The "Needs Alex's yes" column below now means "needs one of those five actions from him", not permission to build.

| Id | Package | Needs Alex's yes | Checks |
|---|---|---|---|
| PP1 | **First slice.** The master switch on the admin config page, plus pure code and tests: the split sum (price, tax, fee, 80 to 20, in whole pence with no rounding leaks), the "may this carry a price" rule (fan-work field, licences, non-commercial catalogue values) returning named reasons, the seller declaration text with its version, the open split line shown to buyers and sellers. No new tables | No | every row of the table in section 4 reproduced; a fan pack, a CC BY-NC asset and a Gaia value each refuse with the right reason; site unchanged with the gate off |
| PP2 | Tables: listings' price, seller accounts, purchases, payout ledger. Empty, locked to the server | **Yes** (migration, run by hand) | existing pages and uploads unaffected; nothing readable from the browser |
| PP3 | Stripe in test mode: checkout session, webhook, purchase recorded, refund and dispute handling | **Yes** (needs a Stripe account and test keys from Alex) | a test purchase, refund and chargeback each leave the ledger correct; a replayed webhook changes nothing |
| PP4 | Paid download: gate checks the purchase, signed receipt attached, pairing route for SSE4 | Yes (storage) | built on maps first, packs when TG15 lands; no purchase, no bytes |
| PP5 | Seller and buyer screens: Stripe onboarding, declaration, price box with refusals and the live split, statement page; price, split line and buy button on the pack page; the test-mode band | Yes (switch) | a fan-marked pack shows no price box |
| PP6 | Moderation: first-sale review, the new report reason, delist-and-refund | Yes | one action delists, refunds and takes back the share |
| PP7 | Monthly payout run with the £25 floor and 30-day hold | Yes | test-mode payout matches the ledger to the penny |
| PP8 | Turn it on: prices on the shelf, buyer and seller terms, footer details | **Yes, and only after the accountant's answers** | |

SSE4 side, small and later: PK1 catalogued objects exported by reference, with the licence flag per catalogue (kernel thread, with X5); PK2 the shelf shows a price and "Buy on Explorers", and Workshop shows "Licensed to" (chart room thread). Neither is needed before PP4.

## 12. To confirm with an accountant

1. That the site is the deemed seller of every pack for VAT, and that creators' shares are a cost of sale to your business.
2. Which of your businesses should run it, and whether a separate company is worth it to keep other people's money apart from your main trade.
3. Registering for the non-Union One Stop Shop: which member state, the cost of the quarterly return, the ten-year record rule.
4. Which countries outside the UK and EU, if any, need registration from the first sale and should be blocked at launch.
5. Paperwork for creators' shares: whether self-billed invoices are needed, and what changes when a creator is VAT-registered or overseas.
6. Whether the UK digital platform reporting rules (reporting sellers' income to HMRC) apply. They cover goods and personal services; downloads look outside them, but it should be said by someone qualified.
7. The wording of the buyer terms, the seller agreement, and the consent box that ends the 14-day cancellation right.

## 13. Rulings (Alex, 21 Nov: "Stripe Connect - yup", "Yes to recs")

Each line below was put as a question with a recommendation. All recommendations are accepted and now stand as rulings. Question 3 holds subject to the registration in 3.4 being in place before EU sales are switched on.

1. **Is "a standard agent" a standard payment provider that also pays the sellers, and is Stripe Connect the one?** *Recommend yes to both.* It is the only mainstream route that takes the payment and pays many creators; the tax-handling providers forbid marketplaces.
2. **Your existing VAT-registered business is the seller on every receipt.** *Recommend yes*, subject to the accountant on which entity. HMRC's rule makes the site the seller in any case.
3. **Sell to EU buyers from the start, with the One Stop Shop registration in place first?** *Recommend yes.* A large share of the audience is there, it is one registration and one return a quarter, and Stripe Tax does the sums. The fallback is UK and rest-of-world first, EU blocked until registered.
4. **What "20% off the top" means.** *Recommend:* tax and card fees come off first, then 20% to the site and 80% to the creator. On a £3 UK sale that is £1.79 to the creator and 45p to the site.
5. **Minimum price £2**, with £3, £5 and £8 suggested. *Recommend yes.*
6. **Payouts monthly, once a creator is owed £25 or more, after a 30-day hold, with Stripe's payout charges coming out of the site's share.** *Recommend yes.*
7. **Selling is by invitation at first** (trusted creators with free work already published). *Recommend yes.*
8. **No copy protection: a signed receipt naming the buyer, and the pack works offline and self-hosted for ever.** *Recommend yes.*
9. **One purchase covers the GM's whole table, and buyers may publish campaigns made with a paid pack but not the pack.** *Recommend yes.*
10. **Refunds:** on request within 14 days if not downloaded, within 30 days if broken or not as described; the site absorbs chargeback fees and takes the sale back from the creator. *Recommend yes.*
11. **Updates to a paid pack are free to earlier buyers.** *Recommend yes.*
12. **The first version of every paid pack is checked by a moderator before it can be sold.** *Recommend yes.*
13. **Gaia:** a paid pack may need the free real-sky pack but never contain its data; exports store catalogued stars by reference; the real-sky pack is served from SSE4's storage, not the shop. *Recommend yes.*
14. *Settled by Alex, 21 Nov:* build all of it now behind the admin master switch, in test mode.
15. *Settled by Alex, 21 Nov:* he takes the cut only and does not sell packs himself; free is always free.
16. **Copyright stays as it is, and the company is named only as operator and seller in the terms, receipts and footer.** *Recommend yes.*

## 14. Sources

- HMRC, "VAT rules for supplies of digital services to consumers", gov.uk (platform rule, EU registration, location evidence), read 21 Nov 2026.
- HMRC, "Register for VAT", gov.uk (£90,000 threshold), read 21 Nov 2026.
- HM Treasury, "Extending VAT online marketplace liability", 28 Oct 2026: goods only, planned for October 2027, does not touch downloads.
- Stripe, UK pricing page (card, Stripe Tax and dispute fees) and Connect pricing page, read 21 Nov 2026.
- Stripe, Managed Payments documentation and eligibility page (3.5% on top of card fees; UK sellers eligible; Connect and marketplaces unsupported), read 21 Nov 2026.
- Paddle, acceptable use policy, updated 13 Apr 2026 (digital marketplaces prohibited), read 21 Nov 2026.
- Lemon Squeezy closing dates, Creem and Whop features: from comparison and vendor marketing pages, not checked at source.
- ESA Gaia licence, as recorded in `SSE4-night-sky-design.md` section 8.
