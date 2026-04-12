import type { Metadata } from "next";
import { Inter, Plus_Jakarta_Sans, Instrument_Serif } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"], variable: "--font-inter" });
const plusJakartaSans = Plus_Jakarta_Sans({ subsets: ["latin"], variable: "--font-plus-jakarta" });
const instrumentSerif = Instrument_Serif({ weight: "400", style: ["normal", "italic"], subsets: ["latin"], variable: "--font-instrument" });

export const metadata: Metadata = {
  title: "Nyxx | Automation that Listens.",
  description: "Turn Any Desktop Task into a Chat Command.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${inter.variable} ${plusJakartaSans.variable} ${instrumentSerif.variable}`}>
      <body className="bg-[#070b0a] font-sans text-white">{children}</body>
    </html>
  );
}