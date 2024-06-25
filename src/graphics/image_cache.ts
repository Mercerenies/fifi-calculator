
import { LruCache } from '../lru_cache.js';

// Cache of recent plot images, so that we don't have to constantly
// re-render them as the stack shuffles about.
export class ImageCache {
  private impl: LruCache<Base64Payload, ImageUrl>;

  constructor(maxCacheSize: number) {
    this.impl = new LruCache(maxCacheSize);
  }

  get(key: Base64Payload): ImageUrl | undefined {
    return this.impl.get(key);
  }

  set(key: Base64Payload, url: ImageUrl) {
    this.impl.set(key, url);
  }
}

export type Base64Payload = string;
export type ImageUrl = string;

export const GLOBAL_IMAGE_CACHE = new ImageCache(10);
