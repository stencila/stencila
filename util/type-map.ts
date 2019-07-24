import { Entity } from '../types'

export type TypeMap<T extends Entity = Entity> = { [key in T['type']]: key }

export type TypeMapGeneric<
  T extends { type: string } & object = { type: string }
> = { [key in T['type']]: key }
