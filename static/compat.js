const root = document.querySelector("[data-compat]");
if (root) {
  const chips = [...root.querySelectorAll("[data-filter]")];
  const search = root.querySelector("[data-search]");
  const rows = [...root.querySelectorAll("tbody tr")];
  const empty = root.querySelector("[data-empty]");
  let status = "";

  const apply = () => {
    const q = search.value.trim().toLowerCase();
    let shown = 0;
    for (const row of rows) {
      const ok = (!status || row.dataset.status === status) && (!q || row.dataset.text.includes(q));
      row.hidden = !ok;
      if (ok) shown++;
    }
    empty.hidden = shown > 0;
  };

  for (const chip of chips) {
    chip.addEventListener("click", () => {
      status = chip.dataset.filter;
      for (const c of chips) c.setAttribute("aria-pressed", String(c === chip));
      apply();
    });
  }
  search.addEventListener("input", apply);
}
