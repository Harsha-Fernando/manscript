(function () {
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

  document.querySelectorAll('main pre').forEach(function (pre, index) {
    const code = pre.querySelector('code');
    if (!code) return;

    const block = document.createElement('div');
    block.className = 'code-block';
    pre.parentNode.insertBefore(block, pre);
    block.appendChild(pre);

    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'code-copy';
    button.setAttribute('aria-label', 'Copy code block ' + (index + 1) + ' to clipboard');
    button.textContent = 'Copy';

    const status = document.createElement('span');
    status.className = 'copy-status';
    status.setAttribute('aria-live', 'polite');
    status.setAttribute('aria-atomic', 'true');

    block.insertBefore(button, pre);
    block.appendChild(status);

    let timer;
    button.addEventListener('click', async function () {
      const copied = await writeClipboard(code.textContent);
      window.clearTimeout(timer);

      if (copied) {
        block.classList.add('is-copied');
        button.textContent = 'Copied';
        status.textContent = 'Code copied to clipboard.';
      } else {
        button.textContent = 'Try again';
        status.textContent = 'Copy failed. Select the code and copy it manually.';
      }

      timer = window.setTimeout(function () {
        block.classList.remove('is-copied');
        button.textContent = 'Copy';
        status.textContent = '';
      }, 2200);
    });
  });
})();
