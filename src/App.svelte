<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { writeText } from '@tauri-apps/plugin-clipboard-manager'
  import {
    containers,
    containersError,
    containersLoading,
    type ContainerInfo,
  } from './lib/containers'

  let busyIds = new Set<string>()
  let copiedId = ''

  const refresh = async () => {
    containersLoading.set(true)
    containersError.set(null)
    try {
      const data = (await invoke('list_containers')) as ContainerInfo[]
      containers.set(data)
    } catch (err) {
      containersError.set(normalizeError(err))
    } finally {
      containersLoading.set(false)
    }
  }

  const runAction = async (action: string, containerId: string) => {
    setBusy(containerId, true)
    containersError.set(null)
    try {
      await invoke(action, { containerId })
      await refresh()
    } catch (err) {
      containersError.set(normalizeActionError(action, err))
    } finally {
      setBusy(containerId, false)
    }
  }

  const copyExec = async (containerId: string) => {
    containersError.set(null)
    try {
      await writeText(`docker exec -it ${containerId} /bin/bash`)
      copiedId = containerId
      window.setTimeout(() => {
        if (copiedId === containerId) {
          copiedId = ''
        }
      }, 2000)
    } catch (err) {
      containersError.set(normalizeError(err))
    }
  }

  const setBusy = (containerId: string, isBusy: boolean) => {
    const next = new Set(busyIds)
    if (isBusy) {
      next.add(containerId)
    } else {
      next.delete(containerId)
    }
    busyIds = next
  }

  const normalizeActionError = (action: string, err: unknown) => {
    const base = normalizeError(err)
    if (action === 'start_container') {
      return `Failed to start container. ${base}`
    }
    if (action === 'stop_container') {
      return `Failed to stop container. ${base}`
    }
    if (action === 'restart_container') {
      return `Failed to restart container. ${base}`
    }
    return base
  }

  const normalizeError = (err: unknown) => {
    if (err instanceof Error && err.message) {
      return err.message
    }
    if (typeof err === 'string') {
      return err
    }
    return 'Unexpected error occurred.'
  }

  const formatName = (container: ContainerInfo) => {
    if (container.name) {
      return container.name
    }
    return container.id.slice(0, 12)
  }

  const shortId = (id: string) => id.slice(0, 12)

  onMount(refresh)
</script>

<main class="app">
  <header class="top-bar">
    <div>
      <p class="kicker">Engine Console</p>
      <h1>Container Control</h1>
      <p class="subtitle">Start, stop, and copy exec commands in one place.</p>
    </div>
    <div class="toolbar">
      <button class="button ghost" on:click={refresh} disabled={$containersLoading}>
        {$containersLoading ? 'Refreshing...' : 'Refresh'}
      </button>
    </div>
  </header>

  {#if $containersError}
    <section class="alert">{$containersError}</section>
  {/if}

  <section class="panel">
    <div class="panel-header">
      <h2>Containers</h2>
      <span class="count">{$containers.length} total</span>
    </div>

    <div class="list">
      {#if $containers.length === 0 && !$containersLoading}
        <div class="empty">No containers found.</div>
      {/if}

      {#each $containers as container, index (container.id)}
        <article class="row" style={`--delay: ${index * 30}ms`}>
          <div class="meta">
            <div class="name">{formatName(container)}</div>
            <div class="detail">
              <span class={`state ${container.state}`}>{container.state}</span>
              <span class="dot"></span>
              <span class="image">{container.image}</span>
              <span class="dot"></span>
              <span class="id">{shortId(container.id)}</span>
            </div>
          </div>
          <div class="actions">
            <button
              class="button"
              on:click={() => runAction('start_container', container.id)}
              disabled={busyIds.has(container.id)}
            >
              Start
            </button>
            <button
              class="button"
              on:click={() => runAction('stop_container', container.id)}
              disabled={busyIds.has(container.id)}
            >
              Stop
            </button>
            <button
              class="button"
              on:click={() => runAction('restart_container', container.id)}
              disabled={busyIds.has(container.id)}
            >
              Restart
            </button>
            <button
              class="button ghost"
              on:click={() => copyExec(container.id)}
              disabled={busyIds.has(container.id)}
            >
              {copiedId === container.id ? 'Copied' : 'Copy exec'}
            </button>
          </div>
        </article>
      {/each}
    </div>
  </section>
</main>
