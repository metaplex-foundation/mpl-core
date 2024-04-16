import { ExternalCheckResult, HookableLifecycleEvent } from "../generated";
import { capitalizeFirstLetter } from "../utils";

export type LifecycleEvent = 'create' | 'update' | 'transfer' | 'burn';

export type LifecycleChecks = { [key in LifecycleEvent]?: ExternalCheckResult }
export type LifecycleChecksContainer = {
  lifecycleChecks?: LifecycleChecks
}

export function lifcycleCheckKeyToEnum (key: keyof LifecycleChecks): HookableLifecycleEvent {
  return HookableLifecycleEvent[capitalizeFirstLetter(key) as keyof typeof HookableLifecycleEvent]
}


export function lifecycleChecksToBase(l: LifecycleChecks): [HookableLifecycleEvent, ExternalCheckResult][] {
  return Object(l).keys().map((key: keyof LifecycleChecks) => [lifcycleCheckKeyToEnum(key), l[key]])
}