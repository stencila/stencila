(function() {
    function rgbToHex(rgb) {
        if (!rgb || rgb === 'transparent') return null;
        const match = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
        if (!match) return null;
        const r = parseInt(match[1]);
        const g = parseInt(match[2]);
        const b = parseInt(match[3]);
        const a = match[4] !== undefined ? parseFloat(match[4]) : 1;
        if (a === 0) return null;
        const hex = '#' + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');
        if (a < 1) {
            const alphaHex = Math.round(a * 255).toString(16).padStart(2, '0');
            return hex + alphaHex;
        }
        return hex;
    }

    const colors = {};
    const elements = document.querySelectorAll('*');
    for (const el of elements) {
        if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
        const cs = getComputedStyle(el);
        for (const prop of ['color', 'backgroundColor', 'borderColor']) {
            const val = cs[prop];
            const hex = rgbToHex(val);
            if (hex) colors[hex] = (colors[hex] || 0) + 1;
        }
    }
    return Object.entries(colors)
        .sort((a, b) => b[1] - a[1])
        .map(([hex, count]) => ({ hex, count }));
})()
