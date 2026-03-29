(function() {
    const root = document.documentElement;
    const styles = getComputedStyle(root);
    const tokens = {};
    for (const sheet of document.styleSheets) {
        try {
            for (const rule of sheet.cssRules) {
                if (rule.style) {
                    for (let i = 0; i < rule.style.length; i++) {
                        const prop = rule.style[i];
                        if (prop.startsWith('--')) {
                            tokens[prop] = styles.getPropertyValue(prop).trim();
                        }
                    }
                }
            }
        } catch (e) { /* cross-origin sheet, skip */ }
    }
    return tokens;
})()
