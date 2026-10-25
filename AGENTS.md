# Coding rules

The rules for writing and changing code in this repository. Read them before your first change.
They apply to the TypeScript and Svelte UI and to the Rust backend. The examples are in
TypeScript.

## 1. Coding standards

Code is read far more often than it is written. Write it for the next person who has to
understand it.

### 1.1 DRY: don't repeat yourself

Every piece of knowledge has one authoritative place in the codebase.

- If you are about to copy a block of code, extract it into a shared function, constant or module
  instead.
- Duplicated logic drifts. When two copies of a rule exist, one gets updated and the other is
  forgotten.
- This goes beyond code: config values, magic numbers, validation rules and type definitions live
  in one place and are referenced everywhere else.
- Don't abstract too early. Two things that look alike today but serve different purposes are
  incidental duplication, and a little duplication is better than the wrong abstraction. The
  test: if the two copies must change together, they should be one thing.

```ts
// Avoid: the same discount rule in two places
const cartTotal = items.reduce((sum, item) => sum + item.price * 0.9, 0);
// In another file:
const invoiceTotal = lines.reduce((sum, line) => sum + line.price * 0.9, 0);

// Prefer: one source of truth
const MEMBER_DISCOUNT_MULTIPLIER = 0.9;

function applyMemberDiscount(price: number): number {
  return price * MEMBER_DISCOUNT_MULTIPLIER;
}
```

### 1.2 Separation of concerns

Each module and function has one clear responsibility.

- **Keep layers apart.** Data access, business logic and presentation don't share a function or
  file. A function that fetches data, transforms it and renders it should be three functions.
- **Functions do one thing.** If you need the word "and" to describe a function, split it.
- **Modules own a domain.** Group code by what it is about, not by what kind of code it is. A
  `billing/` module with its own models, services and validators beats a shared `utils/` bucket.
- **Side effects live at the edges.** Keep core logic pure and testable. Push I/O (network,
  filesystem, processes) to the boundaries.

```ts
// Avoid: fetching, business rules and formatting in one place
async function showUserReport(userId: string) {
  const response = await fetch(`/api/users/${userId}`);
  const user = await response.json();
  const isEligible = user.purchases > 5 && !user.suspended;
  document.getElementById("report").innerHTML = `<p>${user.name}: ${isEligible}</p>`;
}

// Prefer: each concern is separate and testable on its own
async function fetchUser(userId: string): Promise<User> {
  const response = await fetch(`/api/users/${userId}`);
  const responseBody: unknown = await response.json();
  const user = userSchema.parse(responseBody);
  return user;
}

function isEligibleForRewards(user: User): boolean {
  const hasEnoughPurchases = user.purchases > MINIMUM_PURCHASES_FOR_REWARDS;
  const isEligible = hasEnoughPurchases && !user.suspended;
  return isEligible;
}

function renderUserReport(user: User, isEligible: boolean): string {
  const reportHtml = `<p>${user.name}: ${isEligible}</p>`;
  return reportHtml;
}
```

### 1.3 No complex inline returns

Don't return an expression with logic in it. Assign it to a well-named variable, then return the
variable.

- The name documents what the expression means, not only what it computes.
- It gives you a place for a breakpoint or a log line.
- It keeps diffs small when the logic later needs a guard or another step before returning.

```ts
// Avoid: what does this expression represent?
function getPrice(item: Item, user: User): number {
  return item.basePrice * (user.isMember ? 0.9 : 1) + (item.weight > 20 ? 15 : 0);
}

// Prefer: every step is named and inspectable
function getPrice(item: Item, user: User): number {
  const discountMultiplier = user.isMember ? MEMBER_DISCOUNT_MULTIPLIER : 1;
  const discountedPrice = item.basePrice * discountMultiplier;
  const heavyItemSurcharge = item.weight > HEAVY_ITEM_THRESHOLD_KG ? HEAVY_ITEM_FEE : 0;

  const finalPrice = discountedPrice + heavyItemSurcharge;
  return finalPrice;
}
```

Trivial returns (`return null;`, `return [];`, `return isValid;`) are fine. The rule is about
expressions with logic in them.

### 1.4 Braced, multi-line if statements

Single-line `if` statements hide control flow and leave no room for a breakpoint, a log or a
second statement. Always use braces and a new line.

```ts
// Avoid: single-line ifs
if (!user) return null;
if (order.isPaid) sendReceipt(order);
else flagForReview(order);

// Prefer: braced and easy to extend
if (!user) {
  return null;
}

if (order.isPaid) {
  sendReceipt(order);
} else {
  flagForReview(order);
}
```

### 1.5 Maximum nesting depth of 2

Deep nesting is hard to read and test, and it hides the main path. Two levels is the limit. Zero
or one is the goal.

- **Use guard clauses.** Handle invalid and edge cases first and return early, so the main logic
  sits at the top level.
- **Invert conditions to exit early** instead of wrapping the rest of the function in an `if`.
- **Extract nested blocks into named functions.** A loop body or branch complex enough to nest
  further deserves its own name (see 1.6).
- **Flatten loops** with `filter` and `map`, or with an early `continue`.

```ts
// Avoid: four levels deep, with the real work buried
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

// Prefer: filtering and extraction keep it flat
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

Needing a third level is a sign to restructure: extract a function, add a guard clause or rethink
the data flow.

### 1.6 Descriptive names

Names are the main documentation. Every variable, function, type, file and parameter says what it
is without a comment.

- **No abbreviations or single letters**, except in tiny conventional scopes (`i` as a short loop
  index is fine, `usrMgrSvc` never is).
- **Booleans read as questions or states**: `isLoading`, `hasExpired`, `canRetry`, not `flag`,
  `check` or `status2`.
- **Functions are verbs, values are nouns**: `calculateInvoiceTotal()` returns `invoiceTotal`.
- **Include units and context**: `timeoutMs`, `distanceKm`, `retryCount`, not `timeout`,
  `distance`, `n`.
- **If something is hard to name, the design is probably wrong.** A function you can't name
  concisely usually does too much (see 1.2).

```ts
// Avoid
const d = new Date();
const res = getData(u.id);
function proc(x: Purchase[]) {
  return x.reduce((t, p) => t + p.amount, 0);
}

// Prefer
const reportGeneratedAt = new Date();
const purchaseHistory = fetchPurchaseHistory(user.id);
function sumPurchaseAmounts(purchases: Purchase[]) {
  const totalAmount = purchases.reduce((runningTotal, purchase) => {
    return runningTotal + purchase.amount;
  }, 0);
  return totalAmount;
}
```

### 1.7 No `any`, and `unknown` only at trust boundaries

Types are contracts. `any` deletes the contract and `unknown` postpones it. Both hide bugs the
compiler exists to catch.

- **Never use `any`.** It turns off type checking for everything it touches and spreads.
- **Use `unknown` only at trust boundaries** (parsing external JSON, `catch` blocks), and narrow it
  to a concrete type straight away with validation or a type guard.
- **Model the data.** If a shape is awkward to type, define an interface, a union or a generic.
  Awkward types are a design signal, not a reason to opt out.
- Don't silence type errors with `as any`, `@ts-ignore` or `@ts-expect-error`. Fix the type. If a
  library has no types, write a small declaration for the parts you use.

```ts
// Avoid
function handleResponse(data: any) {
  return data.items.map((x: any) => x.name);
}

// Prefer
interface ProductResponse {
  items: Product[];
}

function handleResponse(response: ProductResponse): string[] {
  const productNames = response.items.map((product) => product.name);
  return productNames;
}

// Acceptable: unknown at a trust boundary, narrowed immediately
function parseConfig(rawJson: string): AppConfig {
  const parsed: unknown = JSON.parse(rawJson);
  const validatedConfig = appConfigSchema.parse(parsed);
  return validatedConfig;
}
```

### 1.8 Keep it simple

The best solution is the most boring one that fully solves the problem.

- **Solve the problem in front of you.** Don't build for hypothetical requirements. Extra
  options, plugin systems and generic abstractions added in case they're needed cost now and
  rarely pay off.
- **Prefer obvious constructs.** A plain `if`/`else`, a straightforward loop or a small function
  beats a clever one-liner, a chain of ternaries or an unusual language feature. If a reader has
  to stop and decode it, it's too clever.
- **Fewer moving parts.** Every new layer, wrapper, dependency or pattern has to earn its place.
  No interface with one implementation, factory for one product, or event system where a
  function call would do.
- **Small functions, small files, small changes.** If a solution keeps growing, step back and
  check you are solving the problem that was asked.
- **Delete code gladly.** Removing complexity is as valuable as adding features.

```ts
// Avoid: dense and hard to follow
const getStatus = (u: User) =>
  u?.subscription?.tier
    ? u.subscription.tier === "pro" || u.subscription.expiresAt > Date.now()
      ? "active"
      : "lapsed"
    : "none";

// Prefer: flat and obvious
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

The test for every change: could someone new to the project read it once and understand it?

## 2. Workflow

### 2.1 Explore before acting

Understand the code before changing it.

1. **Read the relevant code paths**: the files involved, the callers of what you are changing, and
   the module around it.
2. **Learn the conventions.** Match how this project structures modules, names things, handles
   errors and writes tests.
3. **Check whether it already exists.** Search for a helper, component or service before writing
   a new one. Reinventing one breaks DRY (1.1) at the project level.
4. **Trace, don't assume.** Follow imports and references to see how data flows and where
   something is used.
5. **Make the smallest change that solves the problem.** Prefer targeted edits to rewrites,
   unless restructuring is the task.
6. **Respect doc comments.** Don't use anything marked `@deprecated` or `#[deprecated]`.

### 2.2 Avoid huge bespoke scripts

- **Use existing tooling first**: the `package.json` scripts, pnpm, Cargo, the test runners and
  the CLIs the project already uses.
- **Work in small steps.** A few short commands you can check as you go beat one long script that
  fails as a black box.
- **Logic worth keeping belongs in the codebase**, placed, named, typed and tested, not in a
  throwaway script.
- **Edit by hand when that is smaller.** Don't write a script to change three files.
- If a substantial script is really needed, explain why first, keep it minimal and remove it
  afterwards.

## 3. Dependencies

Add dependencies with the package manager so you get the latest stable version. Don't type
version numbers into `package.json` or `Cargo.toml` by hand. Pin a specific version only when
there is a stated reason.

| Kind | Command |
|---|---|
| UI or tooling package | `pnpm add <package>`, or `pnpm add -D <package>` for development only |
| Rust crate | `cargo add <crate>`, run in `src-tauri/` |

pnpm is the only JavaScript package manager. Don't use npm, yarn or bun. Commit the updated
`pnpm-lock.yaml` or `src-tauri/Cargo.lock` with the change.

## 4. This project

### 4.1 Checks before a pull request

Run these from the repository root. Run `pnpm fetch-ffmpeg` once first, because the Rust tests
convert real files with the bundled FFmpeg.

```bash
pnpm check
pnpm test:ui
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

All of them must pass with no errors or warnings. CI runs the same checks on Linux, macOS and
Windows, and the end-to-end tests on Linux and Windows. See [Testing](CONTRIBUTING.md#testing).

- Add tests with every change, at the level that fits: Rust unit or conversion tests, UI tests
  or end-to-end tests.
- When a test fails because of a bug, fix the app. Never weaken or skip an assertion.
- Production code never panics or calls `unwrap` on user-controlled input. Errors carry short
  messages a user can act on.

### 4.2 Documentation

- The pages in `docs/` are also rendered in the app's Help page (`src/lib/guide.ts`), so every
  link must work both on GitHub and in the app.
- Link with relative paths to `.md` files, for example `[Saving](saving.md#file-names)`. Links
  between user guide pages open in the app. Links outside `docs/` open on GitHub in the browser.
- Help only shows the pages listed in `PAGE_ORDER` in `src/lib/guide.ts`. Add a new user guide
  page there too.
- Update the docs in the same change as the behaviour they describe.

### 4.3 UI

- Use [shadcn-svelte](https://shadcn-svelte.com) components. Add one with
  `pnpm exec shadcn-svelte add <component>` rather than writing your own.
- Use [Lucide](https://lucide.dev) icons from `@lucide/svelte`. No hand-drawn SVG icons.
- Style with Tailwind and the shadcn theme tokens. No colours outside the tokens.
- Keep the generated files in `src/lib/components/ui/` as the CLI wrote them.

### 4.4 User-facing copy

Applies to UI text, error messages and the docs.

- Short, plain sentences. No filler, no marketing tone, no "simply", "just" or "easily".
- No ellipses and no emoji. Write "Choose files", not "Choose files" followed by an ellipsis.
- Sentence case for headings, buttons and labels.
- In the docs, use the exact labels the UI shows, in bold.
- Errors say what went wrong and, where possible, what to do next.

## 5. Checklist

Before opening a pull request, confirm:

- [ ] I explored the relevant code and followed its conventions
- [ ] No logic or values are duplicated
- [ ] Each function and module has one clear responsibility
- [ ] No complex expressions are returned inline
- [ ] Every `if` is braced and multi-line
- [ ] Nesting never goes deeper than 2 levels
- [ ] Every name is descriptive and unabbreviated
- [ ] No `any`, and `unknown` only at trust boundaries, narrowed immediately
- [ ] The solution is the simplest one that works
- [ ] I used existing tools and small steps, not a large one-off script
- [ ] New dependencies were added with `pnpm add` or `cargo add`
- [ ] The checks in 4.1 pass, and tests cover the change
- [ ] Docs are updated and their links work on GitHub and in the app
