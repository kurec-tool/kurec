import { DynaPuff, Kiwi_Maru } from 'next/font/google';

export const Kiwi_Maru_300 = Kiwi_Maru({
  subsets: ['latin'],
  weight: ['500'],
  display: 'swap',
  variable: '--font-kiwi-maru-300',
  fallback: ['sans-serif'],
});

export const DynaPuff_400 = DynaPuff({
  subsets: ['latin'],
  weight: ['400'],
  display: 'swap',
  variable: '--font-dyna-puff-400',
  fallback: ['"Kiwi Maru"', 'sans-serif'],
});
