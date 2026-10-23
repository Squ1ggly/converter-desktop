# AGENTS.md

Guidance for AI agents (and humans) working in this codebase. Read this before writing or modifying any code.

---

## 1. Coding Standards

We hold a high bar for code quality. Code is read far more often than it is written — optimise for the next person (or agent) who has to understand it.

### 1.1 DRY — Don't Repeat Yourself

Every piece of knowledge should have a single, authoritative representation in the codebase.

- If you find yourself copying a block of code, stop and extract it into a shared function, constant, or module.
- Duplicated logic drifts. When two copies of the same rule exist, one will eventually be updated and the other forgotten.
- This applies beyond code: duplicated config values, magic numbers, validation rules, and type definitions should all live in one place and be referenced everywhere else.
- **However** — don't abstract prematurely. Two things that look similar today but serve different concerns may be incidental duplication. Prefer a little duplication over the wrong abstraction. The test: if the two copies _must_ change together, they should be one thing.

```ts
// ❌ Bad — the same discount rule lives in two places
const cartTotal = items.reduce((sum, item) => sum + item.price * 0.9, 0);
// ...elsewhere...
const invoiceTotal = lines.reduce((sum, line) => sum + line.price * 0.9, 0);

// ✅ Good — one source of truth
const MEMBER_DISCOUNT_MULTIPLIER = 0.9;

function applyMemberDiscount(price: number): number {
  return price * MEMBER_DISCOUNT_MULTIPLIER;
}
```

### 1.2 Separation of Concerns

Each module, class, and function should have one clear responsibility.

- **Keep layers apart.** Data access, business logic, and presentation should not be tangled together in one function or file. A function that fetches data, transforms it, _and_ renders it is three functions wearing a trench coat.
- **Functions do one thing.** If you need the word "and" to describe what a function does, it probably needs splitting.
- **Modules own a domain.** Group code by what it's _about_, not by what it _is_. Prefer `billing/` containing its own models, services, and validators over a giant shared `utils/` bucket.
- **Side effects live at the edges.** Keep core logic pure and testable; push I/O (network, filesystem, database) to the boundaries of the system.

```ts
// ❌ Bad — fetching, business rules, and formatting in one place
async function showUserReport(userId: string) {
  const response = await fetch(`/api/users/${userId}`);
  const user = await response.json();
  const isEligible = user.purchases > 5 && !user.suspended;
  document.getElementById("report").innerHTML =
    `<p>${user.name}: ${isEligible}</p>`;
}

// ✅ Good — each concern is separate and independently testable
async function fetchUser(userId: string): Promise<User> {
  /* data access */
}

function isEligibleForRewards(user: User): boolean {
  return user.purchases > MINIMUM_PURCHASES_FOR_REWARDS && !user.suspended;
}

function renderUserReport(user: User, isEligible: boolean): void {
  /* presentation */
}
```

### 1.3 No Inline Returns

Do not return complex expressions directly. Assign the result to a well-named variable first, then return it.

- A named intermediate variable documents _what_ the expression means, not just what it computes.
- It gives you somewhere to set a breakpoint or add a log line when debugging.
- It keeps diffs clean when the logic later needs a guard clause or transformation before returning.

```ts
// ❌ Bad — what does this expression represent?
function getPrice(item: Item, user: User): number {
  return (
    item.basePrice * (user.isMember ? 0.9 : 1) + (item.weight > 20 ? 15 : 0)
  );
}

// ✅ Good — every step is named and inspectable
function getPrice(item: Item, user: User): number {
  const discountMultiplier = user.isMember ? MEMBER_DISCOUNT_MULTIPLIER : 1;
  const discountedPrice = item.basePrice * discountMultiplier;
  const heavyItemSurcharge =
    item.weight > HEAVY_ITEM_THRESHOLD_KG ? HEAVY_ITEM_FEE : 0;

  const finalPrice = discountedPrice + heavyItemSurcharge;
  return finalPrice;
}
```

Simple early-exit guards (`return null;`, `return [];`, `return isValid;`) are fine — the rule targets _expressions with logic in them_, not trivial returns.

The same rule applies to **inline/single-line `if` statements** — they hide control flow and give you nowhere to add a breakpoint, a log, or a second statement later. Always use braces and a new line:

```ts
// ❌ Bad — inline ifs
if (!user) return null;
if (order.isPaid) sendReceipt(order);
else flagForReview(order);

// ✅ Good — explicit, braced, and extensible
if (!user) {
  return null;
}

if (order.isPaid) {
  sendReceipt(order);
} else {
  flagForReview(order);
}
```

### 1.4 Maximum Nesting Depth: 2 — Prefer Flat Code

Deeply nested code is hard to read, hard to test, and hides the happy path. **Two levels of nesting is the hard ceiling. Zero or one is the goal.**

- **Use guard clauses.** Handle the invalid/edge cases first and return early, so the main logic sits at the top level instead of inside an `if` pyramid.
- **Invert conditions to exit early** rather than wrapping the rest of the function in `if (valid) { ... }`.
- **Extract nested blocks into named functions.** If a loop body or condition branch is complex enough to nest further, it's complex enough to deserve its own name (see 1.5).
- **Flatten loops with array methods or continue-guards.** `filter`/`map` chains, or an early `continue` inside a loop, remove a level of nesting each.

```ts
// ❌ Bad — four levels deep, the actual logic is buried
function processOrders(orders: Order[]) {
  for (const order of orders) {
    if (order.isActive) {
      if (order.items.length > 0) {
        for (const item of order.items) {
          if (item.inStock) {
            shipItem(item);
          }
        }
      }
    }
  }
}

// ✅ Good — guard clauses and extraction keep everything flat
function processOrders(orders: Order[]) {
  const activeOrders = orders.filter((order) => order.isActive);

  for (const order of activeOrders) {
    shipInStockItems(order);
  }
}

function shipInStockItems(order: Order) {
  const inStockItems = order.items.filter((item) => item.inStock);

  for (const item of inStockItems) {
    shipItem(item);
  }
}
```

If you find yourself needing a third level of nesting, that is a signal to restructure — extract a function, add a guard clause, or rethink the data flow. Do not just indent further.

### 1.5 Human-Readable Names for Everything

Names are the primary documentation. Every variable, function, class, file, and parameter should say what it is without needing a comment.

- **No abbreviations or single letters** outside of tiny, conventional scopes (`i` in a short loop index is acceptable; `usrMgrSvc` never is).
- **Booleans read as questions or states:** `isLoading`, `hasExpired`, `canRetry` — not `flag`, `check`, or `status2`.
- **Functions are verbs, values are nouns:** `calculateInvoiceTotal()` returns `invoiceTotal`.
- **Name the units and context:** `timeoutMs`, `distanceKm`, `retryCount` — not `timeout`, `distance`, `n`.
- **If naming something is hard, the design is probably wrong.** A function you can't name concisely is usually doing too many things (see 1.2).

```ts
// ❌ Bad
const d = new Date();
const res = getData(u.id);
function proc(x: any[]) {
  /* ... */
}

// ✅ Good
const reportGeneratedAt = new Date();
const purchaseHistory = fetchPurchaseHistory(user.id);
function summariseMonthlyPurchases(purchases: Purchase[]) {
  /* ... */
}
```

### 1.6 Avoid `any` and `unknown` Types

Types are contracts. `any` deletes the contract; `unknown` defers it. Both hide bugs the compiler exists to catch.

- **Never use `any`.** It silently disables type checking for everything it touches and spreads through the codebase.
- **Avoid `unknown`** except at true trust boundaries (parsing external JSON, catch blocks) — and even there, narrow it to a concrete type immediately with a validation step or type guard.
- **Model your data properly.** If a shape is awkward to type, define an interface, a union, or a generic. Awkwardness in typing is a design signal, not a reason to opt out.
- Do not silence type errors with `as any`, `@ts-ignore`, or `@ts-expect-error`. Fix the underlying type instead. If a third-party library is untyped, write a minimal declaration for the parts you use.

```ts
// ❌ Bad
function handleResponse(data: any) {
  return data.items.map((x: any) => x.name);
}

// ✅ Good
interface ProductResponse {
  items: Product[];
}

function handleResponse(response: ProductResponse): string[] {
  const productNames = response.items.map((product) => product.name);
  return productNames;
}

// ✅ Acceptable use of unknown — narrowed immediately at a trust boundary
function parseConfig(rawJson: string): AppConfig {
  const parsed: unknown = JSON.parse(rawJson);
  const validatedConfig = appConfigSchema.parse(parsed); // e.g. zod
  return validatedConfig;
}
```

### 1.7 Keep It Simple — Don't Over-Complicate

Simplicity is a feature. The best solution is the most boring one that fully solves the problem. Cleverness is a cost, not an achievement.

- **Solve the problem in front of you.** Don't build for hypothetical future requirements (YAGNI). Speculative flexibility — extra config options, plugin systems, generic abstractions "in case we need them" — is complexity paid for now for value that usually never arrives.
- **Prefer boring, obvious constructs.** A plain `if/else`, a straightforward loop, or a simple function beats a clever one-liner, a dense ternary chain, or an exotic language feature. If a reader has to pause and decode it, it's too clever.
- **Fewer moving parts.** Every new layer, wrapper, dependency, or design pattern must earn its place. Don't add an interface with one implementation, a factory for one product, or an event system where a function call would do.
- **Small functions, small files, small changes.** If a solution keeps growing, step back — you may be solving a harder problem than the one that was asked.
- **Delete code gladly.** The simplest code is the code that doesn't exist. Removing complexity is as valuable as adding features.

```ts
// ❌ Bad — clever, dense, and over-engineered for what it does
const getStatus = (u: User) =>
  u?.subscription?.tier
    ? u.subscription.tier === "pro" || u.subscription.expiresAt > Date.now()
      ? "active"
      : "lapsed"
    : "none";

// ✅ Good — boring, flat, and obvious
function getSubscriptionStatus(user: User): SubscriptionStatus {
  if (!user.subscription) {
    return "none";
  }

  const isProTier = user.subscription.tier === "pro";
  const isStillValid = user.subscription.expiresAt > Date.now();

  if (isProTier || isStillValid) {
    return "active";
  }

  return "lapsed";
}
```

The test for every change: **could a new team member read this cold and understand it in one pass?** If not, simplify.

---

## 2. Workflow: Explore Before You Act

Do not start writing code the moment you receive a task. Understanding the codebase first prevents duplicated work, broken conventions, and misplaced changes.

### 2.1 Understand the Project First

Before making any change:

1. **Read the relevant code paths.** Find the files involved in the task and read them properly — not just the function you're changing, but its callers and the module around it.
2. **Learn the existing conventions.** How does this project structure modules? Name things? Handle errors? Write tests? Match the established patterns rather than importing your own.
3. **Check whether it already exists.** Search for existing utilities, helpers, or services before writing new ones. Reinventing an existing helper violates DRY (1.1) at the project level.
4. **Trace, don't assume.** If you're unsure how data flows or where something is used, follow the imports and references. Guessing at behaviour leads to changes that compile but break things.
5. **Make the smallest change that solves the problem.** Prefer targeted edits within existing structures over rewrites and reorganisations, unless restructuring is the actual task.
6. **Respect jsDocs** Eg if a function is marked as @deprecated, don't use it.

### 2.2 Avoid Huge Bespoke Scripts

Do not solve problems by generating large, one-off scripts when smaller, standard approaches exist.

- **Prefer existing tooling.** Use the project's own scripts, package manager commands, test runner, linter, and CLI tools before writing anything custom. Check `package.json` scripts, `Makefile`, or equivalent first.
- **Prefer small, composable steps.** A few short, verifiable commands are better than one 300-line script that does everything invisibly. Small steps can be checked as you go; a monolithic script fails as a black box.
- **Don't script what should be code.** If logic is worth keeping, it belongs in the codebase — properly placed, named, typed, and tested — not in a throwaway script.
- **Don't script what should be manual.** For a one-time, three-file change, just edit the three files. Writing a script to automate a task smaller than the script itself is wasted effort and added risk.
- If a substantial script is genuinely necessary, explain why first, keep it minimal, and clean it up afterwards.

## 3 Dependencies

### 3.1 Use the latest stable version when adding packages

DO NOT add dependencies to the `package.json` by editing the text and hallucinating a version number.

DO use `npm i` to add packages so we get the latest stable version. Only install a specific version if directed to do so.

---

## 4. Quick Checklist

Before submitting any change, confirm:

- [ ] I explored the relevant code and followed existing conventions
- [ ] No logic or values are duplicated — shared code is extracted
- [ ] Each function/module has a single, clear responsibility
- [ ] No complex expressions are returned inline — results are named first
- [ ] No inline/single-line `if` statements — all branches are braced
- [ ] Nesting never exceeds 2 levels — guard clauses and extraction keep code flat
- [ ] Every name is descriptive, unabbreviated, and self-explanatory
- [ ] No `any`; `unknown` only at trust boundaries and narrowed immediately
- [ ] The solution is the simplest one that works — no speculative abstractions or clever tricks
- [ ] I didn't write a large bespoke script where existing tools or small steps would do