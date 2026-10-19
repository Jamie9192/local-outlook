import { useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { X } from 'lucide-react';

type Rect = { left:number; right:number; top:number; bottom:number };
type Size = { width:number; height:number };

export function popupPosition(anchor:Rect, size:Size, viewport:Size) {
 const margin=12, gap=12;
 const width=Math.min(size.width,viewport.width-2*margin),height=Math.min(size.height,viewport.height-2*margin);
 const clamp=(value:number,min:number,max:number)=>Math.max(min,Math.min(value,max));
 let side:'left'|'right'|'above'|'below',left:number,top:number;
 if(anchor.right+gap+width<=viewport.width-margin){side='right';left=anchor.right+gap;top=(anchor.top+anchor.bottom-height)/2;}
 else if(anchor.left-gap-width>=margin){side='left';left=anchor.left-gap-width;top=(anchor.top+anchor.bottom-height)/2;}
 else {
  left=(anchor.left+anchor.right-width)/2;
  side=anchor.bottom+gap+height<=viewport.height-margin||anchor.top-gap-height<margin?'below':'above';
  top=side==='below'?anchor.bottom+gap:anchor.top-gap-height;
 }
 left=clamp(left,margin,viewport.width-width-margin);top=clamp(top,margin,viewport.height-height-margin);
 const arrow=side==='left'||side==='right'?clamp((anchor.top+anchor.bottom)/2-top,18,height-18):clamp((anchor.left+anchor.right)/2-left,18,width-18);
 return {left,top,side,arrow};
}

export default function CalendarPopup({anchor,title,children,onClose}:{anchor:HTMLElement;title:string;children:ReactNode;onClose:(restoreFocus:boolean)=>void}) {
 const panel=useRef<HTMLDivElement>(null),close=useRef<HTMLButtonElement>(null);
 const [position,setPosition]=useState<ReturnType<typeof popupPosition>|null>(null);
 useLayoutEffect(()=>{
  let frame=0;
  const reposition=()=>{
   if(!panel.current||!anchor.isConnected){onClose(false);return;}
   const rect=anchor.getBoundingClientRect(),scroll=anchor.closest('.calendar-scroll')?.getBoundingClientRect();
   if(rect.bottom<0||rect.top>innerHeight||rect.right<0||rect.left>innerWidth||(scroll&&(rect.right<=scroll.left||rect.left>=scroll.right))){onClose(false);return;}
   setPosition(popupPosition(rect,panel.current.getBoundingClientRect(),{width:innerWidth,height:innerHeight}));
  };
  const schedule=()=>{cancelAnimationFrame(frame);frame=requestAnimationFrame(reposition);};
  const outside=(event:Event)=>{const target=event.target as Node;if(!panel.current?.contains(target)&&!anchor.contains(target))onClose(false);};
  const escape=(event:KeyboardEvent)=>{if(event.key==='Escape'){event.preventDefault();onClose(true);}};
  reposition();
  document.addEventListener('pointerdown',outside);document.addEventListener('focusin',outside);document.addEventListener('keydown',escape);
  window.addEventListener('resize',schedule);window.addEventListener('scroll',schedule,true);
  return()=>{cancelAnimationFrame(frame);document.removeEventListener('pointerdown',outside);document.removeEventListener('focusin',outside);document.removeEventListener('keydown',escape);window.removeEventListener('resize',schedule);window.removeEventListener('scroll',schedule,true);};
 },[anchor,title,onClose]);
 useLayoutEffect(()=>{if(position)close.current?.focus({preventScroll:true});},[!!position,title]);
 const arrow=position?.side==='right'?{left:-6,top:position.arrow-6}:position?.side==='left'?{right:-6,top:position.arrow-6}:position?.side==='above'?{bottom:-6,left:position.arrow-6}:{top:-6,left:(position?.arrow??18)-6};
 return <div ref={panel} id="calendar-popup" className="calendar-popup" role="dialog" aria-labelledby="calendar-popup-title" style={position?{left:position.left,top:position.top}:{visibility:'hidden',left:0,top:0}}>
  <span className="calendar-popup-arrow" style={arrow} aria-hidden="true"/>
  <div className="calendar-popup-scroll">
   <header className="calendar-popup-header"><h2 id="calendar-popup-title">{title}</h2><button ref={close} type="button" className="icon-button" aria-label="Close details" onClick={()=>onClose(true)}><X size={18}/></button></header>
   {children}
  </div>
 </div>;
}
