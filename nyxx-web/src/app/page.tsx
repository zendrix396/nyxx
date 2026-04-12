"use client";

import { motion, AnimatePresence } from "framer-motion";
import { Terminal, Rss, Image as ImageIcon, Download, Code2, Network, GitPullRequest, Workflow, ShieldCheck, Zap, ArrowRight, Menu, X } from "lucide-react";
import React, { useEffect, useRef, useState } from "react";
import Hls from "hls.js";

const macros = [
  {
    name: "terminal",
    method: "POST",
    input: "text",
    output: "text",
    os: "Windows 10/11",
    description: "Injects a user-supplied command into a recorded terminal workflow and returns copied output.",
    useCase: "WhatsApp command relay: receive instruction, run locally, copy output, return structured response.",
    icon: <Terminal className="w-6 h-6 text-[#5ed29c]" />,
    url: "/macros/terminal.json"
  },
  {
    name: "hackernews",
    method: "POST",
    input: "text",
    output: "text",
    os: "Windows 10/11",
    description: "Navigates to a news source, captures the page content via recorded browser actions, and returns text.",
    useCase: "Natural request: 'Give me dev news', macro capture, then middleware summary into clean bullet digest.",
    icon: <Rss className="w-6 h-6 text-[#5ed29c]" />,
    url: "/macros/hackernews.json"
  },
  {
    name: "img-gen",
    method: "POST",
    input: "text",
    output: "image (base64)",
    os: "Windows 10/11 (GPU recommended)",
    description: "Feeds prompt text into an image-generation UI flow, copies resulting image, and returns base64 payload.",
    useCase: "Remote text prompt from chat to generated visual without direct UI interaction.",
    icon: <ImageIcon className="w-6 h-6 text-[#5ed29c]" />,
    url: "/macros/img-gen.json"
  },
];

const features = [
  {
    title: "Record once, reuse forever",
    desc: "Capture any workflow (terminal, browser, apps) without writing code. Trigger remotely via WhatsApp or REST.",
    icon: <Code2 className="w-5 h-5" />
  },
  {
    title: "Dynamic input injection",
    desc: "User commands become runtime variables. Nyxx injects inputs exactly where needed during execution.",
    icon: <Zap className="w-5 h-5" />
  },
  {
    title: "Safe by Design",
    desc: "Unlike raw automation, Nyxx uses sandboxed boundaries. LLM cannot execute arbitrary destructive commands.",
    icon: <ShieldCheck className="w-5 h-5" />
  },
  {
    title: "DAG Orchestration (Soon)",
    desc: "Compose complex multi-step pipelines with conditional routing, fan-out execution, and failure recovery.",
    icon: <Workflow className="w-5 h-5" />
  }
];

function VideoBackground() {
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;

    const src = "https://stream.mux.com/tLkHO1qZoaaQOUeVWo8hEBeGQfySP02EPS02BmnNFyXys.m3u8";

    if (Hls.isSupported()) {
      const hls = new Hls({ enableWorker: false });
      hls.loadSource(src);
      hls.attachMedia(video);
      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        video.play().catch(() => {});
      });
      return () => {
        hls.destroy();
      };
    } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
      video.src = src;
      video.addEventListener("loadedmetadata", () => {
        video.play().catch(() => {});
      });
    }
  }, []);

  return (
    <div className="absolute inset-0 z-0 overflow-hidden bg-[#070b0a]">
      <video
        ref={videoRef}
        muted
        autoPlay
        loop
        playsInline
        className="w-full h-full object-cover opacity-60"
      />
      {/* Left dark gradient */}
      <div className="absolute inset-0 bg-gradient-to-r from-[#070b0a] via-[#070b0a]/70 to-transparent" />
      {/* Bottom-up gradient */}
      <div className="absolute inset-0 bg-gradient-to-t from-[#070b0a] via-transparent to-transparent opacity-90" />
    </div>
  );
}

export default function Home() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.15
      }
    }
  };

  const itemVariants = {
    hidden: { opacity: 0, y: 30 },
    visible: { opacity: 1, y: 0, transition: { type: "spring" as const, stiffness: 100, damping: 15 } }
  };

  return (
    <main className="min-h-screen bg-[#070b0a] text-white selection:bg-[#5ed29c]/30">
      
      {/* Global Navigation */}
      <header className="absolute top-0 left-0 right-0 z-50 px-6 py-6 flex justify-between items-center max-w-7xl mx-auto w-full">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded bg-white flex items-center justify-center font-bold text-black font-serif italic text-xl">N</div>
          <span className="font-sans font-bold text-xl tracking-tight text-white">Nyxx</span>
        </div>

        {/* Desktop Menu */}
        <nav className="hidden md:flex items-center gap-8 font-sans text-[16px] text-white/80">
          <a href="#how-it-works" className="hover:text-[#5ed29c] transition-colors uppercase tracking-wide text-sm font-semibold">How it works</a>
          <a href="#catalog" className="hover:text-[#5ed29c] transition-colors uppercase tracking-wide text-sm font-semibold">Catalog</a>
          <a href="#dag" className="hover:text-[#5ed29c] transition-colors uppercase tracking-wide text-sm font-semibold">Roadmap</a>
          <a href="#demo" className="hover:text-[#5ed29c] transition-colors uppercase tracking-wide text-sm font-semibold">Demo</a>
        </nav>

        {/* Mobile Menu Toggle */}
        <button 
          className="md:hidden text-white z-50"
          onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
        >
          {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
        </button>
      </header>

      {/* Mobile Menu Overlay */}
      <AnimatePresence>
        {mobileMenuOpen && (
          <motion.div 
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-[#070b0a] z-40 flex flex-col items-center justify-center gap-8 font-sans text-2xl uppercase tracking-widest font-bold"
          >
            <a href="#how-it-works" onClick={() => setMobileMenuOpen(false)} className="hover:text-[#5ed29c] transition-colors">How it works</a>
            <a href="#catalog" onClick={() => setMobileMenuOpen(false)} className="hover:text-[#5ed29c] transition-colors">Catalog</a>
            <a href="#dag" onClick={() => setMobileMenuOpen(false)} className="hover:text-[#5ed29c] transition-colors">Roadmap</a>
            <a href="#demo" onClick={() => setMobileMenuOpen(false)} className="hover:text-[#5ed29c] transition-colors">Demo</a>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Hero Section */}
      <section className="relative min-h-screen flex flex-col items-center justify-center text-center overflow-hidden pt-20 px-6">
        <VideoBackground />

        {/* Grid System */}
        <div className="absolute inset-0 z-0 pointer-events-none hidden md:block">
          <div className="absolute left-1/4 top-0 bottom-0 w-[1px] bg-white/10" />
          <div className="absolute left-2/4 top-0 bottom-0 w-[1px] bg-white/10" />
          <div className="absolute left-3/4 top-0 bottom-0 w-[1px] bg-white/10" />
        </div>

        {/* Central Glow */}
        <div className="absolute top-[-10%] left-1/2 -translate-x-1/2 w-[800px] h-[300px] z-0 pointer-events-none">
          <svg viewBox="0 0 800 300" xmlns="http://www.w3.org/2000/svg" className="w-full h-full opacity-50" style={{ filter: 'blur(25px)' }}>
            <ellipse cx="400" cy="150" rx="350" ry="100" fill="#5ed29c" />
          </svg>
        </div>
        
        <motion.div 
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 1, ease: "easeOut" }}
          className="relative z-10 max-w-5xl mx-auto flex flex-col items-center"
        >
          {/* The Liquid Glass Card */}
          <div className="liquid-glass-card w-[200px] h-[200px] flex flex-col items-center justify-center text-center p-6 mb-8 translate-y-[-50px]">
            <span className="font-sans text-[14px] text-white/80 mb-4 tracking-widest">[ 2026 ]</span>
            <h2 className="font-sans text-[18px] font-semibold leading-snug mb-3">
              Built with <span className="font-serif italic font-normal text-xl">Rust & AI</span>
            </h2>
            <p className="font-sans text-[11px] text-white/60 leading-relaxed uppercase tracking-wider">
              Native OS Execution
            </p>
          </div>

          <div className="font-jakarta font-bold text-[11px] uppercase tracking-widest text-[#5ed29c] mb-6">
            Hackathon Project: Nyxx
          </div>
          
          <h1 className="font-sans font-extrabold text-[40px] md:text-[72px] uppercase tracking-tighter text-white mb-8 leading-[0.95] max-w-4xl mx-auto">
            TURN ANY DESKTOP TASK INTO A CHAT COMMAND<span className="text-[#5ed29c]">.</span>
          </h1>
          
          <p className="font-sans text-[14px] text-white/70 mb-12 max-w-[512px] mx-auto leading-relaxed">
            Nyxx transforms recorded desktop actions into remotely callable services. 
            Record once, trigger from WhatsApp, let AI route intent, and run deterministic workflows locally.
          </p>

          <a href="#catalog" className="inline-flex items-center gap-2 px-8 py-4 rounded-full bg-[#5ed29c] text-[#070b0a] font-bold uppercase tracking-wider text-sm hover:bg-white transition-all hover:scale-105 active:scale-95">
            Explore Macro Catalog
            <ArrowRight className="w-5 h-5" />
          </a>
        </motion.div>
      </section>

      {/* Features Grid */}
      <section className="py-24 px-6 max-w-7xl mx-auto relative z-10" id="how-it-works">
        <motion.div 
          variants={containerVariants}
          initial="hidden"
          whileInView="visible"
          viewport={{ once: true, margin: "-100px" }}
          className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6"
        >
          {features.map((f, i) => (
            <motion.div key={i} variants={itemVariants} className="liquid-glass-card p-6 hover:border-[#5ed29c]/30 transition-colors group">
              <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center text-white mb-5 group-hover:scale-110 group-hover:text-[#5ed29c] transition-all">
                {f.icon}
              </div>
              <h3 className="text-xl font-bold text-white mb-3 font-serif italic">{f.title}</h3>
              <p className="text-sm text-gray-400 leading-relaxed font-sans">{f.desc}</p>
            </motion.div>
          ))}
        </motion.div>
      </section>

      {/* Story / Architecture */}
      <section className="py-24 px-6 relative overflow-hidden z-10" id="demo">
        <div className="max-w-7xl mx-auto">
          <div className="flex flex-col md:flex-row gap-16 items-center">
            <motion.div 
              initial={{ opacity: 0, x: -40 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="flex-1"
            >
              <h2 className="text-4xl md:text-5xl font-sans tracking-tight font-extrabold text-white mb-6 uppercase">The Magic Behind <br/><span className="font-serif italic font-normal text-[#5ed29c] normal-case text-5xl md:text-6xl">the Execution</span></h2>
              <p className="text-gray-400 text-lg leading-relaxed mb-8 font-sans">
                We separate intent, execution, and intelligence. The middleware LLM translates natural language into a structured API call. The local Rust backend injects variables and runs deterministic macros safely.
              </p>
              
              <div className="space-y-4">
                {[
                  "User sends natural language command in chat.",
                  "Middleware resolves intent into target macro plus typed payload.",
                  "Rust API injects payload into clipboard & executes macro.",
                  "Macro copies result; API returns text/image payload.",
                  "LLM formats result and delivers final response."
                ].map((step, i) => (
                  <div key={i} className="flex gap-4 items-start p-4 rounded-xl liquid-glass-card">
                    <div className="w-8 h-8 shrink-0 rounded-full bg-[#5ed29c]/20 text-[#5ed29c] flex items-center justify-center font-bold text-sm">
                      {i + 1}
                    </div>
                    <p className="text-gray-300 pt-1 leading-snug font-sans">{step}</p>
                  </div>
                ))}
              </div>
            </motion.div>
            
            <motion.div 
              initial={{ opacity: 0, x: 40 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              className="flex-1 w-full"
            >
              <div className="aspect-square w-full max-w-md mx-auto liquid-glass-card p-8 relative overflow-hidden">
                {/* Simulated Terminal UI */}
                <div className="w-full h-full bg-[#030504] rounded-xl border border-white/10 p-4 font-mono text-sm flex flex-col relative z-10">
                  <div className="flex gap-2 mb-4">
                    <div className="w-3 h-3 rounded-full bg-red-500" />
                    <div className="w-3 h-3 rounded-full bg-yellow-500" />
                    <div className="w-3 h-3 rounded-full bg-[#5ed29c]" />
                  </div>
                  <div className="text-gray-500 mb-2">// WhatsApp Listener Active</div>
                  <div className="text-[#5ed29c] mb-2">{"{"}</div>
                  <div className="pl-4 text-gray-300">"intent": "fetch_news",</div>
                  <div className="pl-4 text-gray-300">"macro": "hackernews",</div>
                  <div className="pl-4 text-gray-300">"status": "executing..."</div>
                  <div className="text-[#5ed29c] mb-4">{"}"}</div>
                  
                  <div className="mt-auto border-t border-white/10 pt-4 text-[#5ed29c] animate-pulse">
                    &gt; Success: Payload returned
                  </div>
                </div>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Macro Marketplace */}
      <section id="catalog" className="py-32 px-6 bg-[#070b0a] border-y border-white/5 relative z-10">
        <div className="max-w-7xl mx-auto">
          <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 mb-16">
            <div className="max-w-2xl">
              <h2 className="text-4xl md:text-5xl font-sans tracking-tight font-extrabold text-white mb-4 uppercase">Macro <span className="font-serif italic font-normal text-[#5ed29c] normal-case text-5xl md:text-6xl">Marketplace</span></h2>
              <p className="text-gray-400 text-lg font-sans">
                Download curated starter macros. Uploading your own bundles with signed profiles and security policies is coming soon.
              </p>
            </div>
            <button className="px-6 py-3 rounded-full liquid-glass-card text-gray-400 cursor-not-allowed font-medium whitespace-nowrap uppercase tracking-wider text-sm font-sans">
              Publish Macro (Soon)
            </button>
          </div>

          <motion.div 
            variants={containerVariants}
            initial="hidden"
            whileInView="visible"
            viewport={{ once: true }}
            className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
          >
            {macros.map((m) => (
              <motion.div key={m.name} variants={itemVariants} className="liquid-glass-card p-8 flex flex-col group">
                <div className="flex justify-between items-start mb-6">
                  <div className="w-12 h-12 rounded-xl bg-white/5 flex items-center justify-center border border-white/10">
                    {m.icon}
                  </div>
                  <span className="text-xs font-mono px-3 py-1 rounded-full bg-white/5 text-gray-400">{m.os}</span>
                </div>
                
                <h3 className="text-2xl font-bold font-sans text-white mb-3 capitalize">{m.name}</h3>
                <p className="text-gray-400 text-sm leading-relaxed mb-6 font-sans">{m.description}</p>
                
                <div className="grid grid-cols-2 gap-3 mb-6 bg-black/40 p-4 rounded-xl text-xs font-mono text-gray-300 border border-white/5">
                  <div>
                    <span className="text-gray-500 block mb-1">Method</span>
                    <span className="text-[#5ed29c]">{m.method}</span>
                  </div>
                  <div>
                    <span className="text-gray-500 block mb-1">In / Out</span>
                    <span className="text-white">{m.input} &rarr; {m.output}</span>
                  </div>
                </div>

                <div className="mb-8">
                  <span className="text-[10px] font-bold uppercase tracking-widest text-gray-500 block mb-2 font-jakarta">Use Case</span>
                  <p className="text-sm text-gray-300 leading-relaxed italic border-l-2 border-[#5ed29c]/40 pl-3 font-serif">
                    "{m.useCase}"
                  </p>
                </div>

                <a 
                  href={m.url} 
                  download={`${m.name}.json`}
                  className="mt-auto flex items-center justify-center gap-2 w-full py-4 rounded-full bg-white/10 text-white font-bold hover:bg-[#5ed29c] hover:text-[#070b0a] transition-all uppercase tracking-wider text-sm font-sans"
                >
                  <Download className="w-4 h-4" />
                  Download Config
                </a>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </section>

      {/* DAG Roadmap */}
      <section id="dag" className="py-32 px-6 max-w-7xl mx-auto text-center relative z-10">
        <h2 className="text-4xl md:text-5xl font-sans tracking-tight font-extrabold text-white mb-6 uppercase">DAG <span className="font-serif italic font-normal text-[#5ed29c] normal-case text-5xl md:text-6xl">Orchestration</span></h2>
        <p className="text-gray-400 text-xl max-w-3xl mx-auto mb-16 leading-relaxed font-sans">
          Nyxx is evolving from single macro invocations into composable automation graphs. 
          Deterministic, inspectable, and resumable multi-step desktop automations.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 text-left">
          {[
            { t: "Phase 1: Typed Nodes", d: "Represent each macro as a strongly typed node with explicit JSON contracts." },
            { t: "Phase 2: Conditional Routing", d: "Add branch nodes for intent routing, policy checks, and fallback recovery." },
            { t: "Phase 3: Parallel Execution", d: "Support fan-out branches for multi-source gathering and fan-in merges." },
            { t: "Phase 4: Durable Runs", d: "Checkpoint node state, resume failed runs, and attach execution traces." }
          ].map((phase, i) => (
            <div key={i} className="liquid-glass-card p-8 overflow-hidden group">
              <div className="absolute top-0 right-0 w-32 h-32 bg-[#5ed29c]/10 rounded-full blur-[50px] group-hover:bg-[#5ed29c]/20 transition-all" />
              <div className="relative z-10">
                <div className="text-[#5ed29c] font-serif italic text-2xl mb-2">0{i + 1}</div>
                <h3 className="text-xl font-bold font-sans uppercase tracking-tight text-white mb-3">{phase.t}</h3>
                <p className="text-gray-400 font-sans leading-relaxed">{phase.d}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Footer CTA */}
      <footer className="py-24 border-t border-white/5 text-center px-6 relative z-10 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-transparent to-[#5ed29c]/5 pointer-events-none" />
        <h2 className="text-4xl md:text-5xl font-sans tracking-tight font-extrabold text-white mb-6 uppercase relative z-10">Ready to <span className="font-serif italic font-normal text-[#5ed29c] normal-case text-5xl md:text-6xl">Automate?</span></h2>
        <p className="text-gray-400 text-lg mb-10 max-w-2xl mx-auto font-sans relative z-10">
          We don't just automate tasks—we turn your computer into a programmable service you can call from anywhere.
        </p>
        <div className="flex items-center justify-center gap-2 text-sm text-[#5ed29c] font-jakarta uppercase tracking-widest relative z-10">
          Built for the Hackathon <GitPullRequest className="w-4 h-4 ml-2" />
        </div>
      </footer>
    </main>
  );
}