var cacheName = "egui-template-pwa";
var filesToCache = [
  "./",
  "./index.html",
  "./eframe_paint.js",
  "./eframe_paint_bg.wasm",
];

/**
 * Checks if the current request URL contains #dev
 *
 * @param request The fetch request to check
 * @returns True if the request is for development mode
 */
function isDevMode(request) {
  return request.url.includes("#dev");
}

/* Start the service worker and cache all of the app's content */
self.addEventListener("install", function (e) {
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Serve cached content when offline, except in dev mode */
self.addEventListener("fetch", function (e) {
  // Skip cache in dev mode
  if (isDevMode(e.request)) {
    e.respondWith(fetch(e.request));
    return;
  }

  e.respondWith(
    caches.match(e.request).then(function (response) {
      return response || fetch(e.request);
    })
  );
});
