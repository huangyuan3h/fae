import type { DB } from "../db/types";
import {
  SettingsRepository,
  type ProviderSettingsDTO
} from "../repositories/settings-repository";

export class SettingsService {
  private readonly repo: SettingsRepository;

  constructor(db: DB) {
    this.repo = new SettingsRepository(db);
  }

  getOllama() {
    return {
      baseUrl: this.repo.getOllamaBaseUrl()
    };
  }

  saveOllama(baseUrl: string) {
    this.repo.saveOllamaBaseUrl(baseUrl);
    return { baseUrl };
  }

  getProviders(): ProviderSettingsDTO {
    return this.repo.getProviderSettings();
  }

  saveProviders(payload: ProviderSettingsDTO): ProviderSettingsDTO {
    this.repo.saveProviderSettings(payload);
    return payload;
  }
}
