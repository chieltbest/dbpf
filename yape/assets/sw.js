// SPDX-FileCopyrightText: 2021-2022 eframe_template contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// From: "eframe_template" https://github.com/emilk/eframe_template/tree/main

var cacheName = 'egui-template-pwa';
var filesToCache = [
  './',
  './index.html',
  './eframe_template.js',
  './eframe_template_bg.wasm',
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    caches.match(e.request).then(function (response) {
      return response || fetch(e.request);
    })
  );
});
