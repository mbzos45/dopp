import { writable } from 'svelte/store'

export type ContainerInfo = {
  id: string
  name: string
  image: string
  state: string
  status: string
}

export const containers = writable<ContainerInfo[]>([])
export const containersError = writable<string | null>(null)
export const containersLoading = writable(false)
