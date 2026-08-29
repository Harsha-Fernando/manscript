(function () {
  const hintDefault = 'Click to copy';
  const hintCopied = 'Copied!';

  document.querySelectorAll('main pre').forEach((pre) => {
    const code = pre.querySelector('code');
    if (!code) return;

    const block = document.createElement('div');
    block.className = 'code-block';
    pre.parentNode.insertBefore(block, pre);
    block.appendChild(pre);

    const hint = document.createElement('span');
    hint.className = 'code-copy-hint';
    hint.setAttribute('aria-live', 'polite');
    hint.textContent = hintDefault;
    block.appendChild(hint);

    let timer;

    const copy = async () => {
      const text = code.textContent;

      try {
        await navigator.clipboard.writeText(text);
      } catch {
        const area = document.createElement('textarea');
        area.value = text;
        area.setAttribute('readonly', '');
        area.style.position = 'fixed';
        area.style.left = '-9999px';
        document.body.appendChild(area);
        area.select();
        document.execCommand('copy');
        document.body.removeChild(area);
      }

      block.classList.add('is-copied');
      hint.textContent = hintCopied;
      clearTimeout(timer);
      timer = setTimeout(() => {
        block.classList.remove('is-copied');
        hint.textContent = hintDefault;
      }, 1600);
    };

    block.addEventListener('click', copy);
    pre.setAttribute('tabindex', '0');
    pre.setAttribute('role', 'button');
    pre.setAttribute('aria-label', 'Copy command to clipboard');
    pre.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        copy();
      }
    });
  });
})();
