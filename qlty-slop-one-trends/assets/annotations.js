// Shared by repository and portfolio reports; popovers escape chart clipping.
function setupReportAnnotations({container, getReport, hideTooltip, onDetails, detailsLabel = 'View contributing files'}) {
  const annotationPopover = document.querySelector('#chart-annotation-popover');
  const annotationTooltip = document.querySelector('#annotation-tooltip');
  let annotationView = null;
  let annotationsVisible = true;
  document.querySelector('#annotation-action-label').textContent = detailsLabel;

  function node(tag, text, className) {
    const element = document.createElement(tag);
    if (text !== undefined) element.textContent = text;
    if (className) element.className = className;
    return element;
  }

  function hideAnnotationTooltip() {
    if (annotationTooltip.matches(':popover-open')) annotationTooltip.hidePopover();
  }

  function showAnnotationTooltip(annotation, button) {
    if (!annotationsVisible || annotationPopover.matches(':popover-open')) return;
    hideTooltip();
    annotationTooltip.textContent = annotation.title;
    annotationTooltip.showPopover();
    const anchor = button.getBoundingClientRect();
    const bounds = annotationTooltip.getBoundingClientRect();
    let left = anchor.right + 8;
    let top = anchor.top + (anchor.height - bounds.height) / 2;
    if (left + bounds.width > innerWidth - 12) left = anchor.left - bounds.width - 8;
    if (left < 12) {
      left = Math.max(12, Math.min(anchor.left, innerWidth - bounds.width - 12));
      top = anchor.top - bounds.height - 8;
    }
    annotationTooltip.style.left = `${left}px`;
    annotationTooltip.style.top = `${Math.max(12, Math.min(top, innerHeight - bounds.height - 12))}px`;
  }

  function hideAnnotation() {
    hideAnnotationTooltip();
    if (annotationPopover.matches(':popover-open')) annotationPopover.hidePopover();
    annotationView?.button.setAttribute('aria-expanded', 'false');
    annotationView = null;
  }

  function showAnnotation(annotation, button) {
    const wasOpen = annotationView?.button === button && annotationPopover.matches(':popover-open');
    hideAnnotation();
    hideTooltip();
    if (wasOpen) return;
    annotationView = {annotation, button};
    document.querySelector('#annotation-period').textContent = annotation.label;
    document.querySelector('#annotation-title').textContent = annotation.title;
    document.querySelector('#annotation-body').textContent = annotation.body;
    button.setAttribute('aria-expanded', 'true');
    annotationPopover.showPopover();
    const anchor = button.getBoundingClientRect();
    const bounds = annotationPopover.getBoundingClientRect();
    const left = Math.max(12, Math.min(anchor.left, innerWidth - bounds.width - 12));
    const top = anchor.top >= bounds.height + 24 ? anchor.top - bounds.height - 12 : anchor.bottom + 12;
    annotationPopover.style.left = `${left}px`;
    annotationPopover.style.top = `${Math.max(12, Math.min(top, innerHeight - bounds.height - 12))}px`;
  }

  function renderAnnotations() {
    hideAnnotation();
    container.querySelectorAll('.chart-annotation').forEach(group => group.remove());
    const report = getReport();
    const box = container.querySelector('.chart').viewBox.baseVal;
    for (const [index, annotation] of (report.annotations || []).entries()) {
      const group = node('div', undefined, 'chart-annotation');
      group.hidden = !annotationsVisible;
      const x = (annotation.point[0] - box.x) / box.width * 100;
      const y = (annotation.point[1] - box.y) / box.height * 100;
      const edge = (annotation.lower_edge - box.y) / box.height * 100;
      group.style.setProperty('--point-x', `${x}%`);
      group.style.setProperty('--point-y', `${y}%`);
      group.style.setProperty('--edge-y', `${edge}%`);
      group.style.setProperty('--axis-y', `${(annotation.baseline - box.y) / box.height * 100}%`);
      const point = node('span', undefined, 'annotation-point');
      if (report.values[annotation.period_index].with_file_changes.net > 0) point.classList.add('positive');
      const leader = node('span', undefined, 'annotation-leader');
      point.setAttribute('aria-hidden', 'true');
      leader.setAttribute('aria-hidden', 'true');
      const button = node('button', undefined, 'annotation-marker');
      button.type = 'button';
      button.setAttribute('aria-label', `${annotation.label}: ${annotation.title}`);
      button.setAttribute('aria-haspopup', 'dialog');
      button.setAttribute('aria-controls', 'chart-annotation-popover');
      button.setAttribute('aria-expanded', 'false');
      const number = node('span', String(index + 1), 'annotation-number');
      number.setAttribute('aria-hidden', 'true');
      button.append(number, node('span', undefined, 'button-target'));
      button.addEventListener('pointerenter', event => {
        if (event.pointerType !== 'touch') showAnnotationTooltip(annotation, button);
      });
      button.addEventListener('pointerleave', hideAnnotationTooltip);
      button.addEventListener('focus', () => {
        // Closing a native popover can restore focus during its hide operation.
        // Wait until that operation finishes before opening another popover.
        queueMicrotask(() => {
          if (document.activeElement === button) showAnnotationTooltip(annotation, button);
        });
      });
      button.addEventListener('blur', hideAnnotationTooltip);
      button.addEventListener('keydown', event => {
        if (event.key === 'Escape') hideAnnotationTooltip();
      });
      button.addEventListener('click', () => showAnnotation(annotation, button));
      group.append(leader, point, button);
      container.append(group);
    }
  }

  annotationPopover.addEventListener('toggle', event => {
    if (event.newState === 'closed' && !annotationPopover.matches(':popover-open')) hideAnnotation();
  });
  document.querySelector('#annotation-close').addEventListener('click', () => {
    const button = annotationView?.button;
    hideAnnotation();
    button?.focus({preventScroll: true});
  });
  document.querySelector('#annotation-files').addEventListener('click', () => {
    if (!annotationView) return;
    const {annotation} = annotationView;
    hideAnnotation();
    onDetails(annotation);
  });

  window.addEventListener('resize', hideAnnotation);
  window.addEventListener('scroll', hideAnnotation, {passive: true});
  container.closest('.chart-scroll').addEventListener('scroll', hideAnnotation, {passive: true});

  return {
    hide: hideAnnotation,
    render: renderAnnotations,
    isOpen: () => annotationPopover.matches(':popover-open'),
    setVisible(visible) {
      annotationsVisible = visible;
      hideAnnotation();
      container.querySelectorAll('.chart-annotation').forEach(group => { group.hidden = !visible; });
    }
  };
}
