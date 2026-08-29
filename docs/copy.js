(function () {
  const hintDefault = 'Click to copy';
  const hintCopied = 'Copied!';

  function fallbackCopy(text) {
    const area = document.createElement('textarea');
    area.value = text;
    area.setAttribute('readonly', '');
    area.style.position = 'fixed';
    area.style.inset = '0 auto auto -9999px';
    document.body.appendChild(area);
    area.select();

    let copied = false;
    try {
      copied = document.execCommand('copy');
    } catch (error) {
      copied = false;
    } finally {
      area.remove();
    }
    return copied;
  }

  async function writeClipboard(text) {
    if (navigator.clipboard && window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(text);
        return true;
      } catch (error) {
        // Continue to the legacy fallback when permission is unavailable.
      }
    }
    return fallbackCopy(text);
  }

  document.querySelectorAll('main pre').forEach(function (pre) {
    const code = pre.querySelector('code');
    if (!code) return;

    const block = document.createElement('div');
    block.className = 'code-block';
    pre.parentNode.insertBefore(block, pre);
    block.appendChild(pre);

    const hint = document.createElement('span');
    hint.className = 'code-copy-hint';
    hint.setAttribute('aria-live', 'polite');
    hint.setAttribute('aria-atomic', 'true');
    hint.textContent = hintDefault;
    block.appendChild(hint);

    let timer;

    async function copy() {
      const copied = await writeClipboard(code.textContent);
      window.clearTimeout(timer);

      if (copied) {
        block.classList.add('is-copied');
        hint.textContent = hintCopied;
      } else {
        hint.textContent = 'Copy failed — select manually';
      }

      timer = window.setTimeout(function () {
        block.classList.remove('is-copied');
        hint.textContent = hintDefault;
      }, 1600);
    }

    pre.setAttribute('tabindex', '0');
    pre.setAttribute('role', 'button');
    pre.setAttribute('aria-label', 'Copy command to clipboard');
    pre.addEventListener('click', copy);
    pre.addEventListener('keydown', function (event) {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        copy();
      }
    });
  });
})();
