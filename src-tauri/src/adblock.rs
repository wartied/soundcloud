pub fn get_adblock_script() -> &'static str {
    r#"(function() {
    var BLOCKED = [
        /googlesyndication\.com/,
        /doubleclick\.net/,
        /securepubads/,
        /moatads\.com/,
        /quantserve\.com/,
        /amazon-adsystem\.com/,
        /adsco\.re/,
        /adswizz\.com/,
        /\.ads\./,
        /\/audio-ad/,
        /\/promoted/,
        /pagead/,
        /adservice\.google/,
        /google-analytics\.com/,
        /googletagmanager\.com/,
        /scorecardresearch\.com/,
        /rubiconproject\.com/,
        /pubmatic\.com/,
        /casalemedia\.com/,
        /openx\.net/,
        /criteo\.com/,
        /taboola\.com/,
        /outbrain\.com/,
    ];

    function isBlocked(url) {
        if (!url) return false;
        for (var i = 0; i < BLOCKED.length; i++) {
            if (BLOCKED[i].test(url)) return true;
        }
        return false;
    }

    var _fetch = window.fetch;
    window.fetch = function(input, init) {
        var url = typeof input === 'string' ? input : (input && input.url) || '';
        if (isBlocked(url)) return Promise.resolve(new Response('', { status: 204 }));
        return _fetch.apply(this, arguments);
    };

    var _xhrOpen = XMLHttpRequest.prototype.open;
    XMLHttpRequest.prototype.open = function(method, url) {
        if (isBlocked(url)) {
            this._blocked = true;
            return;
        }
        this._blocked = false;
        return _xhrOpen.apply(this, arguments);
    };
    var _xhrSend = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.send = function() {
        if (this._blocked) return;
        return _xhrSend.apply(this, arguments);
    };

    var AD_SEL = '.sc-ad,.adContainer,[class*="Ad__"],[class*="adContainer"],[class*="promoSlot"],[class*="upsellBanner"],.listenEngagement__promoting';
    var _pending = false;

    function removeAds() {
        var els = document.querySelectorAll(AD_SEL);
        for (var i = 0; i < els.length; i++) els[i].remove();
    }

    function scheduleRemove() {
        if (_pending) return;
        _pending = true;
        requestAnimationFrame(function() { _pending = false; removeAds(); });
    }

    function startObserver() {
        removeAds();
        new MutationObserver(scheduleRemove)
            .observe(document.body, { childList: true, subtree: true });
    }

    var s = document.createElement('style');
    s.textContent = '::-webkit-scrollbar { display: none !important; } html { scrollbar-width: none !important; }';
    (document.head || document.documentElement).appendChild(s);

    if (document.body) startObserver();
    else document.addEventListener('DOMContentLoaded', startObserver);
})();"#
}

pub fn get_track_scraper_js() -> &'static str {
    r#"(function() {
    try {
        if (!window.__scObserverReady) {
            var cb = function() { window.__scDirty = true; };
            var playBtn = document.querySelector('.playControl');
            var badge = document.querySelector('.playbackSoundBadge');
            if (playBtn && badge) {
                new MutationObserver(cb).observe(playBtn, { attributes: true, attributeFilter: ['class'] });
                new MutationObserver(cb).observe(badge, { childList: true, subtree: true, characterData: true, attributes: true });
                window.__scObserverReady = true;
                window.__scDirty = true;
            }
        }

        var now = Date.now();
        if (window.__scObserverReady && !window.__scDirty && (now - (window.__scLastScrape || 0)) < 5000) {
            return null;
        }
        window.__scDirty = false;
        window.__scLastScrape = now;

        var titleEl = document.querySelector('.playbackSoundBadge__titleLink');
        if (!titleEl) return null;

        var title = (titleEl.getAttribute('title') || titleEl.textContent || '').trim();
        if (!title) return null;

        var trackUrl = titleEl.getAttribute('href') || '';
        if (trackUrl && trackUrl.indexOf('http') !== 0)
            trackUrl = 'https://soundcloud.com' + trackUrl;

        var artistEl = document.querySelector('.playbackSoundBadge__lightLink');
        var artist = artistEl ? (artistEl.getAttribute('title') || artistEl.textContent || '').trim() : '';

        var artworkUrl = '';
        var artEl = document.querySelector('.playbackSoundBadge .sc-artwork span');
        if (artEl) {
            var u = (artEl.style.backgroundImage || '').match(/url\("?(.+?)"?\)/);
            if (u) artworkUrl = u[1].replace(/-t\d+x\d+/, '-t500x500');
        }
        if (!artworkUrl) {
            var imgEl = document.querySelector('.playbackSoundBadge .sc-artwork img');
            if (imgEl && imgEl.src) artworkUrl = imgEl.src.replace(/-t\d+x\d+/, '-t500x500');
        }

        var playBtn2 = document.querySelector('.playControl');
        var isPlaying = playBtn2 ? playBtn2.classList.contains('playing') : false;

        var elapsedMs = 0, durationMs = 0;
        function parseTime(s) {
            if (!s) return 0;
            var p = s.trim().split(':');
            return p.length === 3 ? (+p[0]*3600 + +p[1]*60 + +p[2]) * 1000
                 : p.length === 2 ? (+p[0]*60 + +p[1]) * 1000 : 0;
        }
        var timeEl = document.querySelector('.playbackTimeline__timePassed span[aria-hidden="true"]');
        var durEl = document.querySelector('.playbackTimeline__duration span[aria-hidden="true"]');
        if (timeEl) elapsedMs = parseTime(timeEl.textContent);
        if (durEl) durationMs = parseTime(durEl.textContent);

        if (!durationMs) {
            var prog = document.querySelector('.playbackTimeline__progressWrapper [role="progressbar"]');
            if (prog) {
                var pnow = parseFloat(prog.getAttribute('aria-valuenow') || '0');
                var max = parseFloat(prog.getAttribute('aria-valuemax') || '0');
                if (max > 0) { durationMs = max; elapsedMs = pnow; }
            }
        }

        return {
            title: title, artist: artist, artworkUrl: artworkUrl,
            trackUrl: trackUrl, isPlaying: isPlaying,
            elapsedMs: elapsedMs, durationMs: durationMs
        };
    } catch(e) { return null; }
})()"#
}

pub fn get_zoom_js() -> &'static str {
    r#"(function() {
    var L = [25,33,50,67,75,80,90,100,110,125,150,175,200,250,300];
    var saved = parseInt(localStorage.getItem('__sc_zoom') || '100', 10);
    var idx = L.indexOf(saved);
    if (idx < 0) idx = L.indexOf(100);

    function apply() {
        document.documentElement.style.zoom = (L[idx] / 100);
        localStorage.setItem('__sc_zoom', L[idx]);
    }
    if (L[idx] !== 100) apply();

    document.addEventListener('keydown', function(e) {
        if (!e.ctrlKey) return;
        if (e.key === '=' || e.key === '+') {
            e.preventDefault();
            if (idx < L.length - 1) { idx++; apply(); }
        } else if (e.key === '-') {
            e.preventDefault();
            if (idx > 0) { idx--; apply(); }
        } else if (e.key === '0') {
            e.preventDefault();
            idx = L.indexOf(100);
            apply();
        }
    });

    document.addEventListener('wheel', function(e) {
        if (!e.ctrlKey) return;
        e.preventDefault();
        if (e.deltaY < 0 && idx < L.length - 1) { idx++; apply(); }
        else if (e.deltaY > 0 && idx > 0) { idx--; apply(); }
    }, { passive: false });
})()"#
}
