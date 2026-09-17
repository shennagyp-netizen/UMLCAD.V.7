import React from 'react';
import { createRoot } from 'react-dom/client';
import FeatureModelViewer, { loadFeatureBuild } from './FeatureModelViewer';
import type { CompiledModelPackage } from './types';
import './styles.css';

async function loadCompiledModel(): Promise<{ compiled: CompiledModelPackage; buildUrl: string }> {
  const params = new URLSearchParams(window.location.search);
  const packageUrl = params.get('package');
  if (!packageUrl) throw new Error('No compiled model package was supplied.');
  const response = await fetch(packageUrl, { cache: 'no-store' });
  if (!response.ok) throw new Error(`Unable to load compiled model: HTTP ${response.status}.`);
  const compiled = await response.json() as CompiledModelPackage;
  const buildUrl = params.get('build');
  if (!buildUrl) throw new Error('No validated build package was supplied.');
  return { compiled, buildUrl };
}

void loadCompiledModel()
  .then(async ({ compiled, buildUrl }) => {
    const build = await loadFeatureBuild(buildUrl);
    if (build.buildIdentity !== compiled.buildIdentity)
      throw new Error('Compiled model and build package identities do not match.');
    createRoot(document.getElementById('root')!).render(
      <React.StrictMode><FeatureModelViewer compiled={compiled} build={build} /></React.StrictMode>
    );
  })
  .catch((error: unknown) => {
    const root = document.getElementById('root')!;
    root.textContent = error instanceof Error ? error.message : String(error);
  });
