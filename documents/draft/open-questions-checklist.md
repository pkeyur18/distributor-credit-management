# Questions We Need You To Answer
## Distributor Business Volume and Beneficiary Management System

**For:** Siddharth Patel

**From:** Keyur Patel

**Date:** 3 August 2026

---

## How to use this document

Your requirement notes were clear enough that we could work out every calculation and check all five of your examples — the numbers all come out exactly as you wrote them. But a working system needs answers to some things your notes did not cover, and we do not want to guess on your behalf.

There are **22 questions** below. For each one we have written down what we would recommend, and why. **In most cases you only need to tick "Agreed" and move on.** Only stop and think hard about the ones where our suggestion does not match what you had in mind.

- **Section A (Questions 1 to 9)** — we cannot start building until these are answered.
- **Section B (Questions 10 to 17)** — needed before we finalise the design.
- **Section C (Questions 18 to 22)** — can be settled as we go, but easier to decide now.

Please tick your answer under each question and send the document back. If you would rather go through it on a call, that works too.

> ## ⚠️ We have renamed three things — 3 August 2026
>
> | Was called | Is now called | What it actually is |
> |---|---|---|
> | Credit points | **Business Volume** | What you type in directly against one person |
> | Business volume | **Total Business Volume** | That person's own figure **plus** their whole team below |
> | Earned points | **Rewards** | The score they are paid |
>
> **Please note "Business Volume" now means something different to what it meant before.** It used to mean the team total; it now means a person's own figure. The team total is called **Total Business Volume**.
>
> Everything you have already agreed has been rewritten in the new wording, so nothing below contradicts anything you decided. **Your decisions have not changed — only the words have.** The word "points" is no longer used at all; figures are written plainly, as in *"Person A's Business Volume is 300."*

**Where we are: all 22 confirmed. Nothing open, nothing provisional, nothing deferred.** As you settle each one we mark it here rather than deleting it — an answered question stays in the document so we can both refer back to what was agreed and why.

| Badge | Meaning |
|---|---|
| ✅ **CONFIRMED** | Settled. We will build on it. |
| 🟡 **PROVISIONAL** | Agreed, but being checked once more with the client. We will design around it, but it could still move. |
| ⏸️ **DEFERRED** | Parked for now, to be picked up later. |
| ☐ **OPEN** | Still waiting on an answer. |

---

## Decisions confirmed so far

| # | Question | What was agreed | Confirmed on |
|---|---|---|---|
| 1 | Which number do we take the percentage of? | Use the person's **Total Business Volume**, not their own Business Volume. | 3 August 2026 |
| 2 | Does a person earn on their own Business Volume? | **No.** Their own Business Volume still decides which slab they land in, but never generates an earning of its own. | 3 August 2026 |
| 3 | Is own Business Volume always included in Total Business Volume? | **Yes, always** — without exception. | 3 August 2026 |
| 4 | Person P in your Scenarios 4 and 5 | Person P's own Business Volume was left out of those examples for simplicity. The rule is unchanged: own Business Volume is always counted. | 3 August 2026 |
| 5 | What does the monthly reset clear? | *(changed)* **Everything goes to zero** — Business Volume, Total Business Volume and Rewards. Yearly reports are built from the saved monthly records. | 3 August 2026 |
| 6 | What counts as "a month"? | **Calendar month**, and the reset closes the month it belongs to. **Plus, at your request:** a permanent alert that cannot be dismissed until the reset is done. | 3 August 2026 |
| 7 | How do we work out a yearly average? | Divide by the number of months that **actually have figures**, and show that month count on the report. | 3 August 2026 |
| 8 | Below-100 report: average of which number? | *(changed from our suggestion)* Their **own Business Volume**, not Total Business Volume. | 3 August 2026 |
| 9 | Decimal places and rounding | Two decimal places everywhere, rounded only when shown. *(changed)* **Rupee entry removed completely** — Business Volume only, decimals allowed on entry, no rupee figure on any screen. | 3 August 2026 |
| 10 | The one conflicting word | Use the **Business Volume** wording throughout. "Purchase volume" is dropped. | 3 August 2026 |
| 11 | Which number shows on the chart? | *(changed)* Show **name, ID and their own Business Volume** — nothing else. | 3 August 2026 |
| 12 | Should the royalty rate be changeable? | **Yes** — the 1% rate is configurable, like everything else. | 3 August 2026 |
| 13 | Royalty at more than one level? | **Yes** — allowed at every level where the qualifying criteria are met. | 3 August 2026 |
| 14 | Add or remove slab rows? | **Yes** — rows can be added and removed, not just re-numbered. | 3 August 2026 |
| 15 | When should numbers recalculate? | **Immediately** on every Business Volume entry. Expected size: 500 to 5,000 people. | 3 August 2026 |
| 16 | Editing, removing and moving people | Edit freely; mark inactive rather than delete. ⚠️ *Moving was originally allowed — since **blocked outright**, see Additional requirement 4.* | 3 August 2026 |
| 17 | What did you mean by "final discounts"? | *(corrected)* It means **final Rewards**, not a discount. Nothing extra to build. Rupee conversion stays **manual, outside the software**. | 3 August 2026 |
| 18 | Reference numbers, top person, loops | Reference must be an existing active person; top person created once at setup; loop-creating moves blocked. | 3 August 2026 |
| 19 | Who logs in? | **One admin login — you only.** Members never log in. Protected by a 6-digit PIN or complex password (still to choose). | 3 August 2026 |
| 20 | Which fields can be exported? | All fields offered, your four defaults pre-ticked. | 3 August 2026 |
| 21 | Where do backup files go? | Downloaded to your computer **and** kept permanently in the system. | 3 August 2026 |
| 22 | Hierarchy deeper than the setting | **Warn but allow.** | 3 August 2026 |

---

## Additional requirements confirmed — 3 August 2026

These came from the client directly rather than as answers to the questions below. Two of them **change decisions that were previously agreed**, so those are called out clearly.

### 1. Phone numbers must be unique

No two members can share a phone number. Trying to add someone on a number already in use is refused with a clear message.

If the number belongs to someone you previously marked **inactive**, the system recognises them, tells you who it is, and offers to **bring that same person back**. They keep their original 6-digit ID, their place in the hierarchy and all their past records — you never end up with two versions of the same person.

### 2. Member ID numbers are random

Each member gets a **randomly picked, currently unused** 6-digit number between 100000 and 999999. Numbers are not handed out in order, so nobody can tell from an ID when someone joined or how many members exist.

Once a number is taken it is never given to anyone else, even if that member is made inactive — which is what makes bringing them back possible.

*(Small point: we have read "start after 1,00,000" as the normal 6-digit range starting at 100000. If you meant strictly above it, the range becomes 100001 instead. It makes no practical difference.)*

### 3. ⚠️ The monthly reset is now enforced

**Once a month ends, you cannot enter any Business Volume at all until you have closed that month.** The Business Volume entry screen locks and tells you which month is waiting.

> **This changes what we agreed earlier.** When we discussed skipped months, blocking entry was one of the options and it was **not** chosen — we went with a reminder alert instead. The client has now asked for the block. The alert stays, and closing the oldest month first stays; the lock is added on top of both.
>
> **It also cancels one earlier rule.** We had said figures entered after the 1st but before you reset would count into the month just gone. With the lock in place that can no longer happen — there is no window in which to enter them.

### 4. ⚠️ Members can never be transferred

If Person P was added under Person A, **P can never be moved to Person B.** Whoever someone is placed under at the start is permanent.

> **This changes what we agreed earlier.** We had said moving someone was allowed, with already-closed months left untouched. That is now blocked outright.

### 5. A useful side effect of the above

Because nobody can ever be moved, and everyone must be added under someone who already exists, the hierarchy can never tangle itself into a loop. It is guaranteed to stay a clean tree, by design rather than by checking.

### ☐ One new question this raises

Now that entry locks when a month is not closed, a whole month could pass with **no entries at all** — because you were locked out the entire time. That leaves an empty month sitting in the records.

Does an empty month count when we work out yearly averages? We suggest **no** — an empty month gets no record at all and is left out of the average, so it cannot drag anyone's figures down. Please confirm.

---

## Before you start: three different numbers

This is the single most important thing in this document. Your original notes used one word for all three of these, and about half the questions below only make sense once they are kept apart.

| Name | What it means | Example |
|---|---|---|
| **Business Volume** | What you type in directly against one person. Nothing else changes this number. | You add 500 to Person D. Person D's Business Volume is 500. |
| **Total Business Volume** | That person's own Business Volume, **plus** the Total Business Volume already worked out for each person directly below them. | Person D has 500, and A, B, C directly below have 300, 50 and 1,000. Person D's Total Business Volume is 1,850. |
| **Rewards** | The score that person is paid for the month, worked out from the slab difference between them and the people directly below them. | Person D's Rewards come to 35. |

**We only ever add one level down.** Each person directly below has already had their own figure worked out, and that figure already includes their own team. So we never go digging through the whole hierarchy — we just add up the finished figures of the people immediately below. Nothing gets counted twice, and nothing gets missed.

The slab a person falls into is decided by their **Total Business Volume**, not by their Business Volume.

### The slab table, for reference

You will need this for several of the questions.

| Total Business Volume | Slab |
|---|---|
| 0 to 99 | 0% |
| 100 to 399 | 2% |
| 400 to 1,199 | 4% |
| 1,200 to 2,999 | 6% |
| 3,000 to 4,999 | 8% |
| 5,000 to 6,999 | 10% |
| 7,000 to 9,999 | 12% |
| 10,000 and above | 14% |

---

# Section A — Needed before we start building

---

## Question 1 — Which number do we take the percentage of?

> ### ✅ CONFIRMED — 3 August 2026
> **Use the person's Total Business Volume, not their own Business Volume.**
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
When a person earns from someone working below them, do we apply the percentage to that person's **Total Business Volume** (their own Business Volume plus the figure already worked out for each person directly below them), or only to that person's **own Business Volume**?

**Why it matters**
For anyone at the bottom of the hierarchy the two numbers are identical, so four of your five examples do not tell them apart. But for anyone with a team below them the two numbers are very different, and so is the money. This affects every single calculation in the system.

**Example**

Person A has three people below them — B, C and D. Below D there are three more people (p1, p2, p3) holding 250 each.

| Person | Business Volume | Total Business Volume | Slab |
|---|---|---|---|
| B | 1,250 | 1,250 | 6% |
| C | 1,250 | 1,250 | 6% |
| D | 500 | 1,250 (500 + 750 from p1, p2, p3) | 6% |
| A | 500 | 4,250 | 8% |

Person A is on the 8% slab, so the difference against each of B, C and D is 2%.

*If we use Total Business Volume:*
- From B — 2% of 1,250 = **25**
- From C — 2% of 1,250 = **25**
- From D — 2% of 1,250 = **25**
- **Person A earns 75**

*If we use own Business Volume:*
- From B — 2% of 1,250 = **25**
- From C — 2% of 1,250 = **25**
- From D — 2% of 500 = **10**
- **Person A earns 60**

**Our suggestion**
Use the person's **Total Business Volume**.

**Why we suggest it**
Your own Scenario 3 does exactly this — Person A takes 6% of B's figure of 1,250, and 1,250 is B's Total Business Volume, not the figure typed in against B. If we are wrong, everyone with a deep team is paid noticeably more than you intended, so please confirm this one specifically.

**Your answer**
- [x] **Agreed — use Total Business Volume** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 2 — Does a person earn anything on their own Business Volume?

> ### ✅ CONFIRMED — 3 August 2026
> **No — a person does not earn on their own Business Volume.**
> Their own Business Volume still count toward their Total Business Volume and still decide which slab they land in.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
A person's own Business Volume push up their Total Business Volume, and so can push them into a higher slab. But do they also earn a percentage **on those own Business Volume**, on top of what they earn from the people below them?

**Why it matters**
It changes every person's payout, and it changes it most for the people at the top who hold large numbers themselves.

**Example**

Taking your Scenario 1 — Person D holds 500, and is on the 6% slab.

*As your examples work today (no earning on own Business Volume):*
- From A — 12, from B — 3, from C — 20 → **35 Rewards**

*If a person also earned on their own Business Volume:*
- The same 35, **plus** 6% of D's own 500 = 30 → **65 Rewards**

Nearly double.

**Our suggestion**
**No** — a person does not earn on their own Business Volume. Their own Business Volume only decides which slab they land in.

**Why we suggest it**
All five of your examples work this way. In every one of them, the person's own Business Volume is added into the Total Business Volume but never appear as an earning. We are fairly confident this is deliberate, but it is worth one line to confirm, because if it is actually an oversight then every payout in the system is currently too low.

**Your answer**
- [x] **Agreed — no earning on own Business Volume** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 3 — Is a person's own Business Volume always counted in their Total Business Volume?

> ### ✅ CONFIRMED — 3 August 2026
> **Yes — always, without exception.**
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
When we add up a person's Total Business Volume, do we always include their own Business Volume along with the figures coming up from the people directly below them?

**Why it matters**
It decides which slab they land in, which decides every payout above and below them.

**Example**
Your Scenario 1 includes Person D's own 500 in the total of 1,850. Your Scenario 3 also includes Person A's own Business Volume. But Scenarios 4 and 5 add up only the people below — Person P's own Business Volume do not appear anywhere in the sum.

**Our suggestion**
**Yes — always included**, without exception.

**Why we suggest it**
Two of your examples do it explicitly and it is the more natural reading. See Question 4 for the two that seem to disagree.

**Your answer**
- [x] **Agreed — own Business Volume always counted** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 4 — Person P in your Scenarios 4 and 5

> ### ✅ CONFIRMED — 3 August 2026
> **It was left out of the write-up for simplicity.** The rule stands unchanged: a person's own Business Volume is always counted.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
In Scenario 4 you wrote Person P's Total Business Volume as `A + B + C + D = 1,00,000`, and in Scenario 5 as `A + B + C + D + E + F + G = 49,000`. Neither includes any points held by Person P. Was that because Person P genuinely holds nothing, or because it was simply left out to keep the example short?

**Why it matters**
Only to confirm that Question 3's rule holds everywhere. In these two particular examples the answer does not change Person P's slab — Person P is already at the top slab either way — but it does change the numbers that appear in your reports.

**Our suggestion**
It was **left out of the write-up for brevity**. The rule stays: a person's own Business Volume is always counted.

**Why we suggest it**
It is the only reading that keeps all five of your examples consistent with one another. If instead you meant that people at the top of a chain genuinely do not have their own Business Volume counted, that is a real rule change and we need to know now.

**Your answer**
- [x] **Agreed — it was just left out of the example** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 5 — What exactly does the monthly reset clear?

> ### ✅ CONFIRMED — 3 August 2026 *(you chose differently to our suggestion)*
> **Everything goes back to zero** — Business Volume, Total Business Volume and Rewards alike. Nothing at all survives a reset.
> **Before anything is cleared, the month is saved permanently.** Every yearly figure is worked out from those saved monthly records, never from the live numbers.
> Because Rewards are wiped too, that saved record is the **only** thing that proves a month ever happened — which is why the backup cannot be skipped, and why the reset is now enforced.
> This was the last question holding up the technical design. It is now settled.

**What we need to know**
When you press the monthly reset, which numbers go back to zero — only the Business Volume you typed in, or the Rewards as well?

**Why it matters**
This is the biggest one in the document. If Rewards are wiped along with everything else, then there is no record of what anyone earned, and **the yearly average reports you asked for cannot be produced at all.**

**Our suggestion**
The reset clears **only the Business Volume you typed in**. Before anything is cleared, the system saves a permanent, unchangeable snapshot of that month — every person's Business Volume, their Total Business Volume, their slab, and their Rewards. Those snapshots are never deleted, and they are what the yearly reports are built from.

**Why we suggest it**
It is the only way the yearly average and the low-performer report can exist. It also means you can go back and look at any past month whenever you want, and that the backup file is a convenience rather than your only copy.

**Your answer**
- [ ] Agreed — clear typed-in Business Volume only, keep permanent monthly snapshots
- [x] **Different — my answer is:** *clear **everything** — Business Volume, Total Business Volume and Rewards all go to zero. Yearly figures come from the saved monthly records.* *(confirmed 3 August 2026)*

---

## Question 6 — What counts as "a month"?

> ### ✅ CONFIRMED — 3 August 2026 *(agreed, with an addition)*
> **A period is a calendar month, and the reset closes whichever month it belongs to.**
> **You also asked for a permanent reset alert** — see "Added at your request" below, which extends the suggestion.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You asked for the reset to be manual, with a reminder on the 1st. But that means the reset might actually be pressed on the 1st, on the 5th, or not at all that month. What decides which month a set of points belongs to?

**Why it matters**
Every report is grouped by month. If "a month" is loosely defined, the reports drift and stop lining up with your own records. We also need to know what to do if you press reset twice in one month, or skip one entirely.

**Our suggestion**
- A period is a **normal calendar month** — 1st to the last day.
- The reset closes whichever month it belongs to, whenever you actually press it. Pressing it on the 5th of September still closes August.
- Points added between the 1st and the moment you press reset are counted into the **month just gone**, not the new one. The screen will say clearly which month it is about to close, so there is no doubt.
- If a month is skipped entirely, that month simply has no snapshot, and the reports say so rather than showing a zero.

**Why we suggest it**
It keeps your reports matching the calendar, which is what everyone naturally expects, while still leaving the reset fully in your hands. If you would rather a period ran from one reset to the next regardless of dates, that also works, but then your monthly reports will cover uneven stretches of time.

**Added at your request — 3 August 2026**

You agreed with the above, and asked that the software actively chase you rather than relying on you to remember. So:

- Once a month ends, an alert appears telling you that month is waiting to be closed. It names the month, so there is never any doubt which one.
- The alert shows in **two places at once** — a bar across the top of every screen, and an entry in your notification list.
- **It cannot be dismissed or snoozed.** It does not go away when you move to another screen, and it does not go away when you log out and back in. The only thing that clears it is completing the reset.
- If the backup fails or you cancel it, nothing is reset and the alert simply stays up.
- If **more than one month** ends up waiting, the alert lists all of them. You close the **oldest first**, and the next one unlocks once that is done. Each month gets its own backup and its own saved record — they are never merged together.

> **This changes the last bullet of our suggestion above.** We had said a skipped month would simply have no record. With this alert in place a month can no longer be quietly skipped — it stays on the list until you close it, so it still gets its own backup and its own record, just later than usual. That is better for your yearly averages, because no month goes missing.

**Your answer**
- [x] **Agreed — calendar month, reset closes the month it belongs to** *(confirmed 3 August 2026, with the reset alert added above)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 7 — How do we work out a yearly average?

> ### ✅ CONFIRMED — 3 August 2026
> **Divide by the number of months that actually have figures, and show that month count on the report.**
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You asked for each person's yearly average. Do we divide the year's total by 12 always, or by the number of months that person actually has figures for?

**Why it matters**
It decides who lands on your below-100 report. Somebody who joined in October and did well would look like a poor performer if we divide their three good months by twelve.

**Example**

A person joined in October. Their Total Business Volume was 300 in October, 250 in November, 350 in December. Total for the year: 900.

| Method | Calculation | Yearly average | Lands on the below-100 report? |
|---|---|---|---|
| Divide by 12 always | 900 ÷ 12 | **75** | **Yes** |
| Divide by months present | 900 ÷ 3 | **300** | No |

Same person, same performance, opposite conclusion.

**Our suggestion**
Divide by **the number of months that person actually has a snapshot for**. The report will also show that count next to the average, so you can always see that a figure is based on three months rather than twelve.

**Why we suggest it**
Dividing by 12 unfairly punishes anyone who joined part-way through the year, and it also punishes everybody if you happen to skip a reset. Showing the month count alongside means you keep full visibility either way.

**Your answer**
- [x] **Agreed — divide by months actually present, and show that count** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 8 — The below-100 report: average of which number?

> ### ✅ CONFIRMED — 3 August 2026 *(you chose differently to our suggestion)*
> **Use the yearly average of their own Business Volume, not their Total Business Volume.**
> The report will list people based on what they personally brought in, regardless of how their team performed.
> The yearly average export still shows **both** figures — only this report's filter changed.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You asked for a separate report listing people whose yearly average falls below 100. Average of **which** of the three numbers — their Business Volume, their Total Business Volume, or their Rewards?

**Why it matters**
The three produce completely different lists of people. Someone can have a low personal number but a strong team below them, or the reverse.

**Our suggestion**
Use the yearly average of their **Total Business Volume**.

**Why we suggest it**
100 is exactly your first slab threshold. Reading it as Total Business Volume makes the report mean "people who never even reached the lowest slab all year", which is a natural thing to want to look at. If you meant something else — for example people who personally brought in very little regardless of their team — tell us and we will switch it.

**Your answer**
- [ ] Agreed — yearly average of Total Business Volume
- [x] **Different — my answer is:** *use the yearly average of their **own Business Volume**, not Total Business Volume* *(confirmed 3 August 2026)*

---

## Question 9 — Decimal places and rounding

> ### ✅ CONFIRMED — 3 August 2026 *(agreed on decimals, changed on rupees)*
> **Two decimal places everywhere internally, rounded only when a number is shown.** As suggested.
> **Rupee entry is removed completely.** You will enter Business Volume and nothing else — see "Changed at your request" below.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
Two related things. First, should Rewards be kept with decimals or rounded to whole numbers? Second, when you enter an amount in rupees that does not divide evenly by 500, what should happen?

**Why it matters**
Small rounding decisions add up across hundreds of people and twelve months, and if we round at the wrong step the totals stop matching what you would get on a calculator.

**Example**

*Decimals in Rewards:* 2% of a Total Business Volume of 325 comes to 6.5. Rounded to 7, then multiplied across many people and months, the drift becomes visible.

*Rupee entry:* you enter Rs 1,250. At 500 rupees per unit that is 2.5.

| Option | Result | What it costs |
|---|---|---|
| Keep 2.5 | 2.5 | Nothing |
| Round down to 2 | 2 | The person quietly loses Rs 250 of value |
| Round up to 3 | 3 | The person quietly gains Rs 250 of value |
| Reject the entry | Nothing recorded | You have to do the maths yourself before typing |

**Our suggestion**
Keep **two decimal places** everywhere internally, and round only when a number is displayed on screen or in a report. For rupee entry, allow fractional figures — Rs 1,250 becomes 2.5, and the original rupee figure is stored alongside so you can always see what was actually entered.

**Why we suggest it**
Nothing is silently lost or gained, and the totals always match a calculator. Rounding at each individual step is what causes reports to disagree with each other.

**Changed at your request — 3 August 2026**

You agreed on the decimal places, and asked us to drop rupees from the system altogether. So:

- **You enter Business Volume, and nothing else.** There is no rupee box on the Business Volume entry screen and no conversion happening behind it. The rupee worked example above no longer applies — it is kept only as a record of what we considered.
- **The Business Volume box accepts decimals.** You can type `250` or `250.50`. You are never forced to round a figure before entering it.
- **Two decimal places are kept everywhere**, and rounding happens only when a number is displayed. Totals always match a calculator.
- **No rupee figure appears on any screen, report or export.**
- **One exception, and it is invisible in day-to-day use:** the "1 = 500 Rs" setting stays on the settings screen, because your own notes asked for it to be configurable. It is there for the record only — nothing in the system uses it, and no other screen ever shows a rupee amount.

> **This reverses an earlier decision.** At the very start we agreed the Business Volume entry screen would offer both a rupee box and a Business Volume box, with you choosing per entry. That is now replaced by points-only entry. Noted here deliberately so the change is on the record rather than quietly dropped.

**Your answer**
- [ ] Agreed — two decimals internally, allow fractional figures
- [x] **Different — my answer is:** *two decimals as suggested, but remove rupee entry entirely — Business Volume only, decimals allowed on entry, no rupee anywhere on screen* *(confirmed 3 August 2026)*

---

# Section B — Needed before we finalise the design

---

## Question 10 — One word in your notes conflicts with your own rule

> ### ✅ CONFIRMED — 3 August 2026
> **Use "Total Business Volume" everywhere.** The single use of "purchase volume" is dropped.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You asked that the system avoid any wording that suggests trade. But one line in your notes asks for "total purchase volume" on the person detail screen. We need a word to use instead.

**Why it matters**
It appears on screen and in every exported file, so it needs to be a word you are happy for anyone to see.

**Our suggestion**
Keep the **Business Volume** wording throughout — that is, "Business Volume" for a person's own figure and "Total Business Volume" for the team total — and simply drop "purchase volume" from that one line. Neither is on your list of words to avoid, and volume was already the term used everywhere else in your notes, so nothing else has to change.

**Why we suggest it**
It is the smallest possible change and it keeps your own vocabulary. If you would prefer something softer still — "team total" or "group volume" — say so now, because renaming it later means touching every screen and every report.

**Your answer**
- [x] **Agreed — use the Business Volume wording everywhere** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 11 — Which number shows on the hierarchy chart?

> ### ✅ CONFIRMED — 3 August 2026 *(you chose differently to our suggestion)*
> **Show name, ID and their own Business Volume — nothing else.** Re-checked with the client and confirmed.
> One thing to be aware of: the slab a person is on is decided by their Total Business Volume, so a person showing a small own-Business-Volume figure on the chart may still be sitting on a high slab. The chart alone will not explain why.

**What we need to know**
You asked that the chart show only three things per person: name, identification number, and Business Volume. But "Business Volume" could mean their own Business Volume or their Total Business Volume.

**Why it matters**
The chart is the screen you will look at most, and the two numbers tell very different stories about the same person.

**Our suggestion**
Show **Total Business Volume** on the chart, clearly labelled. Their own Business Volume stays on the person detail screen, one click away.

**Why we suggest it**
The whole point of the chart is seeing volume build as it moves up the hierarchy — Total Business Volume is the number that shows that. A person's own Business Volume, on its own, is not very informative in a tree view. This also keeps to your three-fields-only rule.

**Your answer**
- [ ] Agreed — show Total Business Volume on the chart
- [x] **Different — my answer is:** *show **name, ID and their own Business Volume** on the chart, nothing else* *(confirmed 3 August 2026)*

---

## Question 12 — Should the 1% royalty rate be changeable?

> ### ✅ CONFIRMED — 3 August 2026
> **Yes — the royalty rate goes into settings, changeable like everything else.**
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You said the number of qualifying people (3 or more) should be changeable in settings. Should the 1% royalty rate itself also be changeable?

**Why it matters**
Adding it to settings now costs almost nothing. Adding it after the system is built means changing code, testing and redeploying.

**Our suggestion**
**Yes** — put the royalty rate in settings alongside everything else.

**Why we suggest it**
You have asked for every other number in the system to be adjustable. It would be odd for this one to be the exception, and it is the cheapest possible thing to allow for now.

**Your answer**
- [x] **Agreed — make the royalty rate changeable** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 13 — Can royalty be collected at more than one level of the same chain?

> ### ✅ CONFIRMED — 3 August 2026
> **Yes — royalty stacks at every level where the qualifying criteria are met**, each level judged on its own direct people.
> Re-checked with the client and confirmed, with the payout consequence understood — the worked example below shows the same 90,000 attracting royalty twice.

**What we need to know**
Suppose Person P qualifies for royalty. If the person **above** P also has three or more people below them at the top slab, that person collects royalty too — including 1% of P's Total Business Volume, which already contains everything P collected royalty on. Should this stacking be allowed all the way to the top?

**Why it matters**
It directly increases how much you pay out in total. This needs to be a deliberate decision, not something that just happens.

**Example**

Three people — A, B and C — each have a Total Business Volume of 10,000 and are on the top slab. They all work under Person P.

- Person P's Total Business Volume is 30,000. Person P has 3 people at the top slab, so Person P qualifies.
  **Person P collects 1% of 30,000 = 300**

Now Person P has two counterparts, Q and R, in exactly the same position. All three work under Person T.

- Person T's Total Business Volume is 90,000. Person T has 3 people at the top slab, so Person T qualifies.
  **Person T collects 1% of 90,000 = 900**

| Level | Collects | On volume of |
|---|---|---|
| P, Q and R (300 each) | **900** | 90,000 |
| T | **900** | the same 90,000 |
| | **1,800 total** | |

Person A's original 10,000 has had royalty charged on it **twice** — once by P, once by T. In a deeper hierarchy it would be charged at every qualifying level above.

**Our suggestion**
**Yes, allow it at every level**, with each level checked independently against its own direct people.

**Why we suggest it**
It is consistent with your rule that calculations propagate all the way to the top, and it Rewards the people who built the largest structures — which is presumably the intent of having a royalty at all. But please look at the example above and confirm you are comfortable with the total, because this is the one recommendation in this document that costs you money if we have guessed wrong.

**Your answer**
- [x] **Agreed — royalty stacks at every qualifying level** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 14 — Can you add or remove slab rows, or only change the numbers?

> ### ✅ CONFIRMED — 3 August 2026
> **Yes — rows can be added and removed**, not just re-numbered. The top slab is always whichever row has the highest percentage.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You said the slabs should be adjustable, and gave examples of changing the thresholds. Should you also be able to **add a new slab row** or **delete an existing one** — going from seven slabs to eight, or down to five?

**Why it matters**
Allowing rows to be added and removed is a somewhat larger piece of work than just letting the existing seven numbers be edited. It is much cheaper to build in from the start than to add later.

**Our suggestion**
**Allow adding and removing rows.** The system always treats the row with the highest percentage as the top slab — the one that triggers royalty — whatever it happens to be at the time.

**Why we suggest it**
It means your slab structure is never boxed in, and the royalty rule keeps working correctly without anyone having to remember to update it separately.

**Your answer**
- [x] **Agreed — allow adding and removing slab rows** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 15 — When should the numbers recalculate?

> ### ✅ CONFIRMED — 3 August 2026
> **Recalculate immediately**, every time Business Volume is saved. No button to press, no waiting.
> Expected size given as **500 to 5,000 people**, which this handles comfortably.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
When you add Business Volume to somebody, should everybody's Total Business Volume, slab and Rewards update immediately, or should the calculation run only when you ask for it?

**Why it matters**
Adding Business Volume to one person at the bottom changes the numbers for everybody above them, all the way to the top. Doing that instantly is fine at moderate numbers of people, but gets slower as the hierarchy grows.

**Our suggestion**
Recalculate **immediately**, so the moment you save a Business Volume entry, every affected person's figures are correct.

**Why we suggest it**
It matches how you described working — add Business Volume, then look at the screen. Anything else means remembering to press a "recalculate" button, and wondering whether the number in front of you is current.

**One thing we need from you to be sure:** roughly how many people do you expect in the system altogether, and roughly how many Business Volume entries per month? A few hundred people is comfortable; several tens of thousands would change our approach.

Expected number of people: **500 to 5,000** ✅  Points entries per month: _____________ ☐ *(still needed)*

> At 500 to 5,000 people, recalculating instantly is comfortable and needs nothing unusual. We will build it to update only the chain above the person who received points, rather than rebuilding everything each time — you will not see any difference, it just keeps it fast as the hierarchy grows.
>
> **Still needed:** a rough number of Business Volume entries per month. It will not change the answer above, but it tells us how hard the system will be worked.

**Your answer**
- [x] **Agreed — recalculate immediately** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 16 — Editing, removing and moving people

> ### ✅ CONFIRMED — 3 August 2026
> **Edit freely. Mark inactive rather than delete. Moving is allowed, and already-closed months stay frozen exactly as they were.**
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
After somebody has been added, can their details be edited? Can they be removed? And can they be **moved** to work under a different person?

**Why it matters**
Moving somebody changes the Total Business Volume of everybody above both their old and their new position, which changes past payouts. Removing somebody entirely would break every past report they appeared in.

**Our suggestion**
- **Editing details** — allowed at any time (name, phone number, address and so on).
- **Removing** — a person can be marked inactive so they stop appearing in new months, but they are **never permanently deleted**. Their history stays intact.
- **Moving to a different person** — allowed, but months that have already been closed stay exactly as they were. Only the current month and future months use the new position.

**Why we suggest it**
Permanently deleting somebody would silently change last year's reports, and you would have no way of knowing why the numbers no longer match what you saw at the time. Freezing closed months means a past report always shows the same thing every time you open it.

**Your answer**
- [x] **Agreed — edit freely, deactivate rather than delete, closed months frozen** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 17 — What did you mean by "final discounts"?

> ### ✅ CONFIRMED — 3 August 2026 *(wording corrected)*
> **It is not a discount — it means the final Rewards.** That is the score the system already calculates, so **there is no missing feature and nothing extra to build.**
> **The rupee conversion stays outside the software.** You take the final Rewards and work out the rupee value yourself at 1 = 500 Rs. The system will never show a rupee figure. If you want that built in later, we can add it then.
> This was the one place we thought a whole feature might be missing. It was not — it was a wording mix-up in the notes.

**What we need to know**
The opening line of your notes says the system will "calculate final discounts based on calculated points and user hierarchy". But no discount appears anywhere in any of your five examples, and nothing else in your notes mentions one.

**Why it matters**
If a discount is a separate thing that needs calculating and displaying, it is a whole feature that has not been described yet, and we would need to work through it with you properly.

**Our suggestion**
The **slab percentage is the discount** — a person on the 8% slab gets 8%. There is no separate calculation, and nothing extra needs building.

**Why we suggest it**
It is the only reading supported by anything else in your notes. But this is the one question in this document where we might be missing a whole feature, so please answer it properly rather than just ticking agreed.

**Your answer**
- [ ] Agreed — the slab percentage is the discount, nothing separate
- [x] **Different — my answer is:** *the wording should be "final Rewards", not "final discounts". Nothing separate to build. The rupee conversion is done manually, outside the software, and must not appear in the application.* *(confirmed 3 August 2026)*

---

# Section C — Can be settled as we design

---

## Question 18 — Reference numbers, the top person, and preventing loops

> ### ✅ CONFIRMED — 3 August 2026
> **All three as suggested** — reference must be an existing active person, the top person is created once at setup, and any move that would put someone under their own team is blocked.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
Three small related points. Must the reference number entered for a new person always be somebody who already exists? How does the very first person — the one at the top — get created, given that a reference number is required for everyone? And what stops somebody being accidentally moved to sit underneath one of their own team?

**Why it matters**
Any of these going wrong produces a hierarchy that cannot be calculated at all.

**Our suggestion**
- The reference number must match an **existing, active** person. Anything else is rejected at the point of entry, with a clear message.
- The single top person is created **once during initial setup**, as a special step, without a reference number. After that the option is not available again — matching your rule that the top level can never grow beyond one person.
- If somebody is moved, the system **blocks** any move that would place them underneath their own team, and explains why.

**Your answer**
- [x] **Agreed** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 19 — Who logs in?

> ### ✅ CONFIRMED — 3 August 2026 *(agreed, with detail added)*
> **One administrator login — yours only.** Members never log in and have no access.
> **Protected by either a 6-digit PIN or a complex password** — you will decide which. Both are catered for until then.
>
> **One thing we must build either way:** a limit on failed attempts, locking the account after too many wrong tries. A 6-digit PIN is only a million combinations, which a computer can work through very quickly if it is allowed unlimited guesses. This single login guards every member's name, phone number and address, so the attempt limit is not optional — it is what makes a PIN safe enough to use at all.

**What we need to know**
Your notes do not mention logging in at all. Is this a single system that only you use, or will other people need their own logins? And will the members themselves ever log in to see their own figures?

**Why it matters**
Member logins in particular are a significant addition — it means every person needs an account, a password, a way to reset it, and a screen showing only their own data.

**Our suggestion**
**One administrator login — yours.** Members do not log in and have no access.

**Why we suggest it**
Nothing you have described needs member access. Everything happens through you. This can be added later if you want it, but it would be a separate piece of work rather than a small adjustment, so it is worth saying now if you think you will want it.

**Your answer**
- [x] **Agreed — single administrator login only** *(confirmed 3 August 2026)*, protected by a 6-digit PIN or complex password — ⏸️ **still to choose which**
- [ ] Different — my answer is: _______________________________________________

---

## Question 20 — Which fields can be added to exports?

> ### ✅ CONFIRMED — 3 August 2026
> **All fields offered, with your four defaults pre-ticked.**
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You said name, identification number, phone number and Business Volume should always be exported, and that you want to choose what else goes in. Which additional fields should be available to choose from?

**Our suggestion**
Make **everything** available, with your four defaults pre-ticked. The full list would be: email address, address, reference number (who they work under), the name of the person they work under, their level in the hierarchy, number of people directly below them, Total Business Volume, slab percentage, Rewards, royalty earned, and their joining date.

**Why we suggest it**
There is no real cost to offering all of them, and it saves coming back to us every time you want a column that was not on the list.

**Your answer**
- [x] **Agreed — offer all fields, four defaults pre-ticked** *(confirmed 3 August 2026)*
- [ ] Anything to add or remove: _______________________________________________

---

## Question 21 — Where do the backup files go?

> ### ✅ CONFIRMED — 3 August 2026
> **Downloaded to your computer, and a copy kept permanently inside the system.** Nothing is ever auto-deleted.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You said the monthly reset cannot happen without a backup being taken first. Where should that file end up, and for how long should it be kept?

**Why it matters**
You have made the backup a hard condition of resetting, which tells us it matters to you. If the only copy is a file sitting in a downloads folder, that protection is thinner than it looks.

**Our suggestion**
The file downloads to your computer **and** a copy is kept permanently inside the system, where you can re-download any past month at any time. Nothing is ever automatically deleted.

**Why we suggest it**
It means a lost or overwritten download is an inconvenience rather than a problem. Combined with the permanent monthly snapshots from Question 5, your history is safe in two independent places.

**Your answer**
- [x] **Agreed — download plus a permanent copy kept in the system** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

## Question 22 — What happens if the hierarchy gets deeper than the setting allows?

> ### ✅ CONFIRMED — 3 August 2026
> **Warn, but allow.** You are never blocked from adding a real person because of a settings value.
> Settled — no further action needed. Kept here in full for future reference.

**What we need to know**
You said the depth of the hierarchy is adjustable in settings. If somebody tries to add a person deeper than that setting allows, should the system refuse, warn, or just allow it?

**Our suggestion**
**Warn, but allow.** A clear message appears explaining that the setting is being exceeded, and you can carry on if you want to.

**Why we suggest it**
This matches the decision already taken on the level widths — the 9, 6 and 3 figures are treated as guidance rather than hard limits, because your own examples do not stick to them. Treating depth the same way keeps the system consistent, and means you are never blocked from adding a real person because of a setting.

**Your answer**
- [x] **Agreed — warn but allow** *(confirmed 3 August 2026)*
- [ ] Different — my answer is: _______________________________________________

---

# Summary — tick as you go

| # | Question | Section | Status |
|---|---|---|---|
| 1 | Which number do we take the percentage of? | A | ✅ **Confirmed 3 Aug 2026** |
| 2 | Does a person earn on their own Business Volume? | A | ✅ **Confirmed 3 Aug 2026** |
| 3 | Are own Business Volume always in the Total Business Volume? | A | ✅ **Confirmed 3 Aug 2026** |
| 4 | Person P in your Scenarios 4 and 5 | A | ✅ **Confirmed 3 Aug 2026** |
| 5 | What does the monthly reset clear? | A | ✅ **Confirmed 3 Aug 2026** *(changed)* |
| 6 | What counts as "a month"? | A | ✅ **Confirmed 3 Aug 2026** |
| 7 | How do we work out a yearly average? | A | ✅ **Confirmed 3 Aug 2026** |
| 8 | Below-100 report: average of which number? | A | ✅ **Confirmed 3 Aug 2026** *(changed)* |
| 9 | Decimal places and rounding | A | ✅ **Confirmed 3 Aug 2026** *(changed)* |
| 10 | The one conflicting word | B | ✅ **Confirmed 3 Aug 2026** |
| 11 | Which number shows on the chart? | B | ✅ **Confirmed 3 Aug 2026** *(changed)* |
| 12 | Should the royalty rate be changeable? | B | ✅ **Confirmed 3 Aug 2026** |
| 13 | Royalty at more than one level? | B | ✅ **Confirmed 3 Aug 2026** |
| 14 | Add or remove slab rows? | B | ✅ **Confirmed 3 Aug 2026** |
| 15 | When should numbers recalculate? | B | ✅ **Confirmed 3 Aug 2026** |
| 16 | Editing, removing and moving people | B | ✅ **Confirmed 3 Aug 2026** |
| 17 | What did you mean by "final discounts"? | B | ✅ **Confirmed 3 Aug 2026** *(corrected)* |
| 18 | Reference numbers, top person, loops | C | ✅ **Confirmed 3 Aug 2026** |
| 19 | Who logs in? | C | ✅ **Confirmed 3 Aug 2026** |
| 20 | Which fields can be exported? | C | ✅ **Confirmed 3 Aug 2026** |
| 21 | Where do backup files go? | C | ✅ **Confirmed 3 Aug 2026** |
| 22 | Hierarchy deeper than the setting | C | ✅ **Confirmed 3 Aug 2026** |

---

## What is still outstanding

**All 22 questions are answered and confirmed. Nothing is blocking the technical design any more.**

Three small items remain, none of which hold anything up:

- A rough number of **Business Volume entries per month** (Question 15). It will not change any decision, it just tells us how hard the system will be worked.
- **PIN or password** for the login (Question 19). Whichever you choose, the failed-attempt lockout gets built.
- ☐ **Whether an empty month counts toward yearly averages** — newly raised by the enforced reset. See the end of "Additional requirements confirmed" above. We suggest it should not count.

**Now closed:**

- **Question 5**, the last blocker, is settled — everything resets to zero, and yearly figures come from the saved monthly records.
- **Question 17** was the one place we thought an entire feature might be missing. It was a wording mix-up — "final discounts" meant **final Rewards**, which the system already calculates. Nothing extra to build.
- **Questions 11 and 13** were re-checked with the client and confirmed as they stood.

---

*Once this comes back, we can begin the technical design. Questions 1 to 9 are the ones holding it up.*
