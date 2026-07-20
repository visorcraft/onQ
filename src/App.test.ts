import { expect, it, vi } from 'vitest';
import { mount, tick, unmount } from 'svelte';

const { invokeMock, openMock, writeTextMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  openMock: vi.fn(),
  writeTextMock: vi.fn(),
}));
vi.mock('@tauri-apps/api/core', () => ({ invoke: invokeMock }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openMock }));

import App from './App.svelte';
import { globalShortcut } from '$lib/stores/globalShortcut';
import { navigate } from '$lib/stores/navigation';
import { tutorialStep, tutorialVisible } from '$lib/stores/tutorial';

function resetUi() {
  navigate('home');
  tutorialStep.set(0);
  tutorialVisible.set(false);
  document.body.replaceChildren();
}

it('opens the tutorial after vault creation and opens About from help', async () => {
  const recoveryPhrase =
    'legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title';
  resetUi();
  const target = document.createElement('div');
  document.body.append(target);
  tutorialStep.set(0);
  tutorialVisible.set(false);
  invokeMock.mockReset();
  openMock.mockReset();
  writeTextMock.mockReset();
  writeTextMock.mockResolvedValue(undefined);
  Object.defineProperty(navigator, 'clipboard', {
    configurable: true,
    value: { writeText: writeTextMock },
  });
  openMock.mockResolvedValue('/tmp/tutorial-test-vault');
  invokeMock.mockImplementation((command: string, args?: { key?: string }) => {
    if (command === 'open_last_vault') {
      return Promise.resolve({
        path: null,
        opened: false,
        needsPassword: false,
        needsRecovery: false,
      });
    }
    if (command === 'setup_new_vault') {
      return Promise.resolve({ recoveryPhrase });
    }
    if (command === 'get_app_setting' && args?.key === 'tutorial_completed') {
      return Promise.resolve('false');
    }
    if (command === 'get_app_setting' && args?.key === 'last_opened_prompt') {
      return Promise.resolve('');
    }
    return Promise.resolve(undefined);
  });

  const component = mount(App, { target });
  await vi.waitFor(() => {
    expect(
      Array.from(document.querySelectorAll('button')).some(
        (button) => button.textContent?.trim() === 'Create new vault',
      ),
    ).toBe(true);
  });

  expect(invokeMock).not.toHaveBeenCalledWith('get_app_setting', {
    key: 'tutorial_completed',
  });

  const createVault = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Create new vault',
  );
  if (!(createVault instanceof HTMLButtonElement)) {
    throw new Error('Create new vault button missing');
  }
  createVault.click();

  await vi.waitFor(() => {
    expect(document.body.textContent).toContain('Protect your vault');
  });
  const withoutPassword = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Create without password',
  );
  if (!(withoutPassword instanceof HTMLButtonElement)) {
    throw new Error('Create without password button missing');
  }
  withoutPassword.click();

  await vi.waitFor(() => {
    expect(document.querySelector('textarea[aria-label="Recovery phrase"]')).not.toBeNull();
  });

  expect(document.querySelector('input[type="checkbox"]')).toBeNull();
  expect(invokeMock).toHaveBeenCalledWith('setup_new_vault', {
    path: '/tmp/tutorial-test-vault',
    masterPassword: null,
  });
  const copy = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Copy',
  );
  if (!(copy instanceof HTMLButtonElement)) throw new Error('Copy button missing');
  copy.click();
  await vi.waitFor(() => {
    expect(writeTextMock).toHaveBeenCalledWith(recoveryPhrase);
    expect(document.querySelector('[role="status"]')?.textContent).toBe('Copied!');
  });

  const acknowledgement = document.querySelector('input[type="checkbox"]');
  if (!(acknowledgement instanceof HTMLInputElement)) {
    throw new Error('Recovery acknowledgement missing');
  }
  acknowledgement.click();
  await tick();

  const continueButton = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Continue',
  );
  if (!(continueButton instanceof HTMLButtonElement)) {
    throw new Error('Recovery continue button missing');
  }
  continueButton.click();

  await vi.waitFor(() => {
    expect(document.querySelector('[role="dialog"]')?.textContent).toContain(
      'Open your prompt palette',
    );
  });
  const tutorialChecks = invokeMock.mock.calls.filter(
    ([command, args]) =>
      command === 'get_app_setting' &&
      (args as { key?: string } | undefined)?.key === 'tutorial_completed',
  );
  expect(tutorialChecks).toHaveLength(1);

  const skip = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Skip tutorial',
  );
  if (!(skip instanceof HTMLButtonElement)) throw new Error('Skip button missing');
  skip.click();
  await vi.waitFor(() => expect(document.querySelector('[role="dialog"]')).toBeNull());

  const aboutBtn = document.querySelector('button[aria-label="About onQ"]');
  if (!(aboutBtn instanceof HTMLButtonElement)) throw new Error('About button missing');
  aboutBtn.click();
  await tick();

  await vi.waitFor(() => {
    expect(document.body.textContent).toContain('Licenses & Credits');
    expect(document.body.textContent).toMatch(/What's inside|Hybrid search/);
  });

  await unmount(component);
});

it('opens the remembered vault on launch', async () => {
  resetUi();
  const target = document.createElement('div');
  document.body.append(target);
  tutorialVisible.set(false);
  globalShortcut.set('Meta+Space');
  invokeMock.mockReset();
  invokeMock.mockImplementation((command: string) => {
    if (command === 'open_last_vault') {
      return Promise.resolve({
        path: '/tmp/remembered-vault',
        opened: true,
        needsPassword: false,
        needsRecovery: false,
      });
    }
    if (command === 'get_app_setting') return Promise.resolve('');
    return Promise.resolve(undefined);
  });

  const component = mount(App, { target });
  await vi.waitFor(() => {
    expect(document.querySelector('.hero h1')?.textContent).toBe('onQ');
  });
  expect(document.body.textContent).not.toContain('Welcome to onQ');
  expect(document.body.textContent).toContain('Press Meta+Space to begin');

  await unmount(component);
  globalShortcut.set('');
});

it('creates a master-password vault without a recovery phrase', async () => {
  resetUi();
  const target = document.createElement('div');
  document.body.append(target);
  tutorialVisible.set(false);
  invokeMock.mockReset();
  openMock.mockReset();
  openMock.mockResolvedValue('/tmp/password-vault');
  invokeMock.mockImplementation((command: string) => {
    if (command === 'open_last_vault') {
      return Promise.resolve({
        path: null,
        opened: false,
        needsPassword: false,
        needsRecovery: false,
      });
    }
    if (command === 'setup_new_vault') return Promise.resolve({ recoveryPhrase: null });
    if (command === 'get_app_setting') return Promise.resolve('');
    return Promise.resolve(undefined);
  });

  const component = mount(App, { target });
  await vi.waitFor(() => {
    expect(document.body.textContent).toContain('Create new vault');
  });
  const create = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Create new vault',
  );
  if (!(create instanceof HTMLButtonElement)) throw new Error('Create button missing');
  create.click();

  await vi.waitFor(() => {
    expect(document.querySelector('#master-password')).not.toBeNull();
  });
  for (const selector of ['#master-password', '#confirm-password']) {
    const input = document.querySelector(selector);
    if (!(input instanceof HTMLInputElement)) throw new Error(`${selector} missing`);
    input.value = 'correct horse battery staple';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }
  await tick();

  const protectedCreate = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Create with password',
  );
  if (!(protectedCreate instanceof HTMLButtonElement)) {
    throw new Error('Create with password button missing');
  }
  protectedCreate.click();

  await vi.waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith('setup_new_vault', {
      path: '/tmp/password-vault',
      masterPassword: 'correct horse battery staple',
    });
    expect(document.querySelector('.hero h1')?.textContent).toBe('onQ');
  });
  expect(document.querySelector('textarea[aria-label="Recovery phrase"]')).toBeNull();

  await unmount(component);
});

it('offers manual recovery when the remembered vault key is missing', async () => {
  resetUi();
  const target = document.createElement('div');
  document.body.append(target);
  tutorialVisible.set(false);
  invokeMock.mockReset();
  invokeMock.mockImplementation((command: string) => {
    if (command === 'open_last_vault') {
      return Promise.resolve({
        path: '/tmp/recovery-vault',
        opened: false,
        needsPassword: false,
        needsRecovery: true,
      });
    }
    if (command === 'recover_vault') return Promise.resolve(undefined);
    if (command === 'get_app_setting') return Promise.resolve('');
    return Promise.resolve(undefined);
  });

  const component = mount(App, { target });
  await vi.waitFor(() => {
    expect(document.body.textContent).toContain('Encryption key missing from system keychain.');
  });
  expect(document.querySelector('#vault-credential')).toBeNull();

  const recoveryButton = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Recover with recovery phrase',
  );
  if (!(recoveryButton instanceof HTMLButtonElement)) {
    throw new Error('Recovery button missing');
  }
  recoveryButton.click();
  await tick();

  const phrase = document.querySelector('#vault-credential');
  if (!(phrase instanceof HTMLTextAreaElement)) throw new Error('Recovery input missing');
  phrase.value =
    'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art';
  phrase.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();

  const unlock = Array.from(document.querySelectorAll('button')).find(
    (button) => button.textContent?.trim() === 'Recover',
  );
  if (!(unlock instanceof HTMLButtonElement)) throw new Error('Recover button missing');
  unlock.click();

  await vi.waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith('recover_vault', {
      path: '/tmp/recovery-vault',
      recoveryPhrase: phrase.value,
    });
    expect(document.querySelector('.hero h1')?.textContent).toBe('onQ');
  });

  await unmount(component);
});

it('asks for the master password when the remembered vault uses one', async () => {
  resetUi();
  const target = document.createElement('div');
  document.body.append(target);
  tutorialVisible.set(false);
  invokeMock.mockReset();
  invokeMock.mockImplementation((command: string) => {
    if (command === 'open_last_vault') {
      return Promise.resolve({
        path: '/tmp/password-vault',
        opened: false,
        needsPassword: true,
        needsRecovery: false,
      });
    }
    if (command === 'unlock_vault') {
      return Promise.resolve({
        path: '/tmp/password-vault',
        opened: true,
        needsPassword: false,
        needsRecovery: false,
      });
    }
    if (command === 'get_app_setting') return Promise.resolve('');
    return Promise.resolve(undefined);
  });

  const component = mount(App, { target });
  await vi.waitFor(() => {
    expect(document.body.textContent).toContain('Enter master password');
  });

  const password = document.querySelector('#vault-credential');
  if (!(password instanceof HTMLInputElement)) throw new Error('Password input missing');
  password.value = 'correct horse battery staple';
  password.dispatchEvent(new Event('input', { bubbles: true }));
  await tick();

  const unlockForm = password.closest('form');
  if (!(unlockForm instanceof HTMLFormElement)) throw new Error('Unlock form missing');
  unlockForm.requestSubmit();

  await vi.waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith('unlock_vault', {
      path: '/tmp/password-vault',
      masterPassword: password.value,
    });
    expect(document.querySelector('.hero h1')?.textContent).toBe('onQ');
  });

  await unmount(component);
});
