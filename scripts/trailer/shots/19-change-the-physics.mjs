// Change the physics: from a value's why? into the flow that made it; a bad edit is refused, a good one is
// checked, its effect shown, adopted, and the world's card says its physics changed.
export default {
  id: "19-change-the-physics",
  title: "Change the physics: a rule is a diagram you can edit",
  feature: "Why? to flow editor, change request, validation, adopt (U3)",
  trailers: ["physics", "features"],
  async setup(page) {
    await page.locator('.area-chart .body[aria-label^="Tarn"]').first().click();
    await page.locator("section[aria-label='Tarn']").waitFor();
  },
  async run(page, d) {
    const card = page.locator("section[aria-label='Tarn']");
    await d.hold(700);
    // To the Build desk, at Engine depth: where rules are changed.
    await d.click(page.getByRole("button", { name: "Build", exact: true }).first());
    const engine = page.getByRole("button", { name: "Engine", exact: true }).first();
    if (await engine.isVisible().catch(() => false)) await d.click(engine);
    await d.hold(600);
    await d.click(card.locator(".field").filter({ hasText: /^surface/i }).getByRole("button", { name: /why/i }));
    const why = page.locator("[role=region][aria-label^='Why']");
    await why.locator(".formula").waitFor();
    await d.hold(1800);
    await d.click(why.getByRole("button", { name: "Open the flow" }));
    const editor = page.locator("section[aria-label='Flow editor']");
    await editor.waitFor();
    await page.waitForFunction(() => /\d/.test(document.querySelector("[data-kind='planet.surface_temperature'] .val")?.textContent ?? ""));
    await d.hold(2200);
    await d.click(editor.locator("[aria-label^='Explain node']").getByRole("tab", { name: "Engine" }));
    await d.hold(1500);
    const param = editor.locator("#p-teq-redistribution");
    await d.click(param);
    await param.fill("");
    await d.type("7", 200);
    await param.press("Enter");
    const request = page.locator(".area-card [role=region][aria-label^='Request']");
    await request.getByText("Rejected").waitFor();
    await d.glide(request, 700);
    await d.hold(2000);
    await d.click(param);
    await param.fill("");
    await d.type("0.5", 200);
    await param.press("Enter");
    await request.getByText("Validated").waitFor();
    await d.glide(request.locator("table.effect").first(), 700);
    await d.hold(2200);
    await d.click(request.getByRole("button", { name: "Adopt" }));
    await page.waitForFunction(() => document.querySelector("[data-stack-hash]")?.getAttribute("data-physics") === "campaign");
    await d.hold(2500);
  },
};
