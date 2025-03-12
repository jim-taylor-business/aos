var cacheName = "aos";
var filesToCache = [
  "./",
  "./manifest.json",
  "./pkg/aos.wasm",
  "./pkg/aos.js",
  "./pkg/aos.css",
  "./public/favicon.ico",
  "./public/favicon.png",
  "./public/icons.svg",
  "./public/lemmy.svg",
];

self.addEventListener("install", function (event) {
  // console.log("Installing");
  const preCache = async () => {
    get_cache().then(function (cache) {
      cache.keys().then(function (requests) {
        for (let request of requests) {
          cache.delete(request);
        }
      });
      cache.addAll(filesToCache.map(url => new Request(url, { credentials: "same-origin" })));
    })
  };
  event.waitUntil(preCache);
});

self.addEventListener("message", function (messageEvent) {
  // console.log("Message");
  if (messageEvent.data === "skipWaiting") {
    // console.log("Service-worker received skipWaiting event");
    self.skipWaiting();
  }
});

self.addEventListener("fetch", function (e) {
  // console.log("Fetch");
  // e.respondWith(cache_then_network(e.request));
});

async function get_cache() {
  return caches.open(cacheName);
}

async function cache_then_network(request) {
  const cache = await get_cache();
  return cache.match(request).then(
    (cache_response) => {
      if (!cache_response) {
        return fetch_from_network(request, cache);
      } else {
        console.log("cache");
        return cache_response;
      }
    },
    (reason) => {
      return fetch_from_network(request, cache);
    }
  );
}

function fetch_from_network(request, cache) {
  // return fetch(request, { cache: "force-cache" }).then(
  return fetch(request).then(
    (net_response) => {
      // console.log("Network");
      return net_response;
    },
    (reason) => {
      // console.error("Network fetch rejected. Falling back to ./. Reason: ", reason);
      return cache.match("./").then(function (cache_root_response) {
        return cache_root_response;
      });
    }
  )
}
