(function(selectors) {
    // Helper function to convert rgb()/rgba() to hex
    function rgbToHex(rgb) {
        if (!rgb || rgb === 'transparent') return null;

        // Match rgb() or rgba()
        const match = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
        if (!match) return null;

        const r = parseInt(match[1]);
        const g = parseInt(match[2]);
        const b = parseInt(match[3]);
        const a = match[4] !== undefined ? parseFloat(match[4]) : 1;

        // If fully transparent, return null
        if (a === 0) return null;

        const hex = '#' + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');

        // Include alpha if not fully opaque
        if (a < 1) {
            const alphaHex = Math.round(a * 255).toString(16).padStart(2, '0');
            return hex + alphaHex;
        }

        return hex;
    }

    const result = {
        css: {},
        box_info: {},
        counts: {},
        dom_context: {},
        text: {},
        errors: []
    };

    for (const sel of selectors) {
        // Count elements
        const elements = document.querySelectorAll(sel);
        result.counts[sel] = elements.length;

        if (elements.length === 0) {
            continue;
        }

        // Get first element for measurements
        const el = elements[0];

        // Computed styles
        const cs = getComputedStyle(el);
        result.css[sel] = {
            // Spacing
            paddingTop: cs.paddingTop,
            paddingBottom: cs.paddingBottom,
            paddingLeft: cs.paddingLeft,
            paddingRight: cs.paddingRight,
            marginTop: cs.marginTop,
            marginBottom: cs.marginBottom,
            marginLeft: cs.marginLeft,
            marginRight: cs.marginRight,
            // Typography
            fontSize: cs.fontSize,
            lineHeight: cs.lineHeight,
            color: cs.color,
            colorHex: rgbToHex(cs.color),
            fontFamily: cs.fontFamily,
            fontWeight: cs.fontWeight,
            textAlign: cs.textAlign,
            textDecoration: cs.textDecoration,
            letterSpacing: cs.letterSpacing,
            textTransform: cs.textTransform,
            whiteSpace: cs.whiteSpace,
            // Display
            display: cs.display,
            visibility: cs.visibility,
            opacity: cs.opacity,
            // Backgrounds
            backgroundColor: cs.backgroundColor,
            backgroundColorHex: rgbToHex(cs.backgroundColor),
            backgroundImage: cs.backgroundImage,
            backgroundSize: cs.backgroundSize,
            backgroundPosition: cs.backgroundPosition,
            // Borders
            borderWidth: cs.borderWidth,
            borderColor: cs.borderColor,
            borderColorHex: rgbToHex(cs.borderColor),
            borderRadius: cs.borderRadius,
            borderStyle: cs.borderStyle,
            borderTopWidth: cs.borderTopWidth,
            borderRightWidth: cs.borderRightWidth,
            borderBottomWidth: cs.borderBottomWidth,
            borderLeftWidth: cs.borderLeftWidth,
            // Layout
            position: cs.position,
            top: cs.top,
            right: cs.right,
            bottom: cs.bottom,
            left: cs.left,
            zIndex: cs.zIndex,
            overflow: cs.overflow,
            overflowX: cs.overflowX,
            overflowY: cs.overflowY,
            minHeight: cs.minHeight,
            maxWidth: cs.maxWidth,
            // Flexbox
            gap: cs.gap,
            justifyContent: cs.justifyContent,
            alignItems: cs.alignItems,
            flexDirection: cs.flexDirection,
            // Visual effects
            boxShadow: cs.boxShadow,
            transform: cs.transform,
            filter: cs.filter
        };

        // Bounding box
        const rect = el.getBoundingClientRect();
        result.box_info[sel] = {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height
        };

        // Text content (truncate if very long)
        const text = el.textContent || '';
        result.text[sel] = text.length > 200 ? text.substring(0, 200) + '...' : text;

        // DOM ancestor chain (Element.parentElement only — does not cross
        // shadow DOM boundaries, iframes, or document fragments)
        const domAncestors = [];
        let parent = el.parentElement;
        const MAX_ANCESTORS = 20;
        while (parent && parent.tagName !== 'BODY' && domAncestors.length < MAX_ANCESTORS) {
            const entry = { tag: parent.tagName.toLowerCase() };
            if (parent.id) entry.id = parent.id;
            const cls = Array.from(parent.classList);
            if (cls.length > 0) entry.classes = cls;
            domAncestors.push(entry);
            parent = parent.parentElement;
        }
        if (parent && parent.tagName === 'BODY') {
            domAncestors.push({ tag: 'body' });
        }

        // Stencila-specific schema attributes
        const depthAttr = el.getAttribute('depth');
        const depthParsed = depthAttr !== null ? Number(depthAttr) : NaN;
        const depthVal = Number.isInteger(depthParsed) && depthParsed >= 0 ? depthParsed : undefined;

        result.dom_context[sel] = {
            tagName: el.tagName.toLowerCase(),
            id: el.id || undefined,
            classes: Array.from(el.classList),
            schemaAncestors: el.getAttribute('ancestors') || undefined,
            schemaDepth: (depthVal !== undefined && !isNaN(depthVal)) ? depthVal : undefined,
            domAncestors: domAncestors
        };
    }

    return result;
})
