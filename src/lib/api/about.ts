import { invoke } from '@tauri-apps/api/core';

export type AboutInfo = {
  appName: string;
  version: string;
  license: string;
  repository: string;
  gitSha: string;
  description: string;
  tagline: string;
  platform: string;
};

export type LicenseDocMeta = {
  id: string;
  title: string;
  subtitle: string;
};

export type CrateCredit = {
  name: string;
  version: string;
  license: string;
  repository: string;
};

export type NpmPackageCredit = {
  name: string;
  version: string;
  license: string;
  repository: string;
  role: string;
};

export type RuntimeComponent = {
  name: string;
  notes: string;
  licenses: string;
  spdxId: string;
  projectUrl: string;
};

export type CreditsData = {
  crates: CrateCredit[];
  packages: NpmPackageCredit[];
  runtime: RuntimeComponent[];
  crateCount: number;
  packageCount: number;
  runtimeCount: number;
};

export async function aboutInfo(): Promise<AboutInfo> {
  return invoke<AboutInfo>('about_info_cmd');
}

export async function licenseDocs(): Promise<LicenseDocMeta[]> {
  return invoke<LicenseDocMeta[]>('license_docs_cmd');
}

export async function licenseDocument(id: string): Promise<string> {
  return invoke<string>('license_document_cmd', { id });
}

export async function creditsData(): Promise<CreditsData> {
  return invoke<CreditsData>('credits_data_cmd');
}

export async function runtimeLicenseText(spdxId: string): Promise<string> {
  return invoke<string>('runtime_license_text_cmd', { spdxId });
}
