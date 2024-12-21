import { getConfig } from '@/lib/config';
import { getKvsValue } from '@/lib/nats';
import { type NextRequest, NextResponse } from 'next/server';

export async function GET(
  req: NextRequest,
  { params }: { params: { hash: string } },
) {
  const { hash } = await params;
  const config = getConfig();
  console.log('hash:', hash);
  const value = await getKvsValue(`${config.prefix}-ogp`, hash);
  if (value) {
    return new NextResponse(value.value, {
      headers: { 'Content-Type': 'image/webp' },
    });
  }
  return NextResponse.json({ message: 'Not Found' }, { status: 404 });
}
