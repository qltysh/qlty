function setupChartOptions({annotationsAvailable, onChange = () => {}, onOpen = () => {}, onExport = () => {}}) {
  const button = document.querySelector('#chart-options-button');
  const menu = document.querySelector('#chart-options');
  const annotations = document.querySelector('#show-annotations');
  const exportSvg = document.querySelector('#export-svg');
  annotations.hidden = !annotationsAvailable;
  annotations.setAttribute('aria-checked', String(annotationsAvailable));

  menu.addEventListener('toggle', event => {
    const open = event.newState === 'open';
    button.setAttribute('aria-expanded', String(open));
    if (!open) return;
    onOpen();
    const anchor = button.getBoundingClientRect();
    const bounds = menu.getBoundingClientRect();
    menu.style.left = `${Math.max(12, Math.min(anchor.right - bounds.width, innerWidth - bounds.width - 12))}px`;
    menu.style.top = `${Math.max(12, Math.min(anchor.bottom + 8, innerHeight - bounds.height - 12))}px`;
    (annotations.hidden ? exportSvg : annotations).focus({preventScroll: true});
  });
  menu.addEventListener('keydown', event => {
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    event.preventDefault();
    const items = [...menu.querySelectorAll('[role^="menuitem"]:not([hidden])')];
    const index = items.indexOf(document.activeElement);
    items[(index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length].focus();
  });
  exportSvg.addEventListener('click', () => {
    menu.hidePopover();
    button.focus({preventScroll: true});
    onExport();
  });
  button.addEventListener('keydown', event => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      menu.showPopover();
    }
  });
  annotations.addEventListener('click', () => {
    const visible = annotations.getAttribute('aria-checked') !== 'true';
    annotations.setAttribute('aria-checked', String(visible));
    onChange(visible);
    menu.hidePopover();
    button.focus({preventScroll: true});
  });
  const close = () => {
    if (menu.matches(':popover-open')) menu.hidePopover();
  };
  window.addEventListener('resize', close);
  window.addEventListener('scroll', close, {capture: true, passive: true});
}
