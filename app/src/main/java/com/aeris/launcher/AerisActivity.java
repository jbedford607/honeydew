package com.aeris.launcher;

import android.app.*;
import android.app.role.RoleManager;
import android.content.*;
import android.content.pm.LauncherActivityInfo;
import android.content.pm.LauncherApps;
import android.graphics.*;
import android.graphics.drawable.Drawable;
import android.os.*;
import android.provider.Settings;
import android.text.*;
import android.text.format.DateFormat;
import android.view.*;
import android.view.inputmethod.InputMethodManager;
import android.widget.*;
import android.graphics.drawable.GradientDrawable;
import java.util.*;
import java.text.*;

public final class AerisActivity extends Activity {
  static final int[] COLORS={0xff159cbe,0xfff47b19,0xff21ac80};
  static final int[] TOP={0xffc9f3ff,0xffffeed8,0xffddfff0};
  static final int[] BOT={0xffdbeaff,0xffffd9c7,0xffd2edf6};
  final ArrayList<Item> all=new ArrayList<>();
  final ArrayList<Item> filtered=new ArrayList<>();
  final ArrayList<String> favorites=new ArrayList<>();
  final ArrayList<String> dock=new ArrayList<>();
  SharedPreferences prefs;
  LauncherApps launcher;
  FrameLayout root;
  FrameLayout overlay;
  EditText search;
  GridView drawerGrid;
  BaseAdapter adapter;
  TextView clock,date,bat;
  int theme=0,accent=COLORS[0],ink=0xff163a4d,secondary=0xff537785;
  boolean drawer=false;
  Handler handler=new Handler(Looper.getMainLooper());
  Runnable ticker=new Runnable(){public void run(){refreshClock();handler.postDelayed(this,30000);}};
  class Item{
    LauncherActivityInfo info;String id,name,pkg;Drawable icon;
    Item(LauncherActivityInfo a){info=a;id=a.getComponentName().flattenToString();pkg=a.getComponentName().getPackageName();name=a.getLabel().toString();icon=a.getIcon(0);}
  }
  @Override public void onCreate(Bundle state){
    super.onCreate(state);
    prefs=getSharedPreferences("aeris",0);
    launcher=(LauncherApps)getSystemService(Context.LAUNCHER_APPS_SERVICE);
    theme=prefs.getInt("theme",0);accent=COLORS[theme];
    getWindow().setStatusBarColor(Color.TRANSPARENT);
    getWindow().setNavigationBarColor(Color.TRANSPARENT);
    getWindow().getDecorView().setSystemUiVisibility(0x100|0x400|0x200|View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR|View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR);
    getWindow().setNavigationBarContrastEnforced(false);
    loadApps();loadPins();render();handler.postDelayed(this::requestHome,900);
  }
  @Override protected void onResume(){super.onResume();handler.removeCallbacks(ticker);handler.post(ticker);loadApps();loadPins();if(root!=null&&!drawer)render();}
  @Override protected void onPause(){handler.removeCallbacks(ticker);super.onPause();}
  @Override public void onBackPressed(){if(drawer)closeDrawer();else super.onBackPressed();}
  int d(float v){return (int)(v*getResources().getDisplayMetrics().density+.5f);}
  GradientDrawable shape(int color,float r){GradientDrawable gd=new GradientDrawable();gd.setColor(color);gd.setCornerRadius(d(r));return gd;}
  Drawable glass(float radius,boolean strong){
    final int bright=strong?245:220;
    GradientDrawable g=new GradientDrawable(GradientDrawable.Orientation.TOP_BOTTOM,new int[]{Color.argb(bright,255,255,255),Color.argb(strong?208:173,228,249,255),Color.argb(strong?236:199,255,255,255)});
    g.setCornerRadius(d(radius));g.setStroke(d(1),0xc6ffffff);return g;
  }
  LinearLayout vertical(){LinearLayout l=new LinearLayout(this);l.setOrientation(1);return l;}
  LinearLayout horizontal(){LinearLayout l=new LinearLayout(this);l.setOrientation(0);l.setGravity(Gravity.CENTER_VERTICAL);return l;}
  LinearLayout.LayoutParams lp(int w,int h){return new LinearLayout.LayoutParams(w<0?w:d(w),h<0?h:d(h));}
  LinearLayout.LayoutParams margins(int w,int h,int l,int t,int r,int b){LinearLayout.LayoutParams a=lp(w,h);a.setMargins(d(l),d(t),d(r),d(b));return a;}
  TextView label(String s,float size,int color,boolean bold){
    TextView v=new TextView(this);v.setText(s);v.setTextColor(color);v.setTextSize(size);
    v.setTypeface(Typeface.create(bold?"sans-serif-medium":"sans-serif",Typeface.NORMAL));v.setGravity(Gravity.CENTER_VERTICAL);
    v.setIncludeFontPadding(false);return v;
  }
  TextView button(String s,Runnable go){
    TextView v=label(s,19,accent,true);v.setGravity(Gravity.CENTER);v.setBackground(glass(17,false));v.setElevation(d(3));
    v.setOnClickListener(w->{w.performHapticFeedback(HapticFeedbackConstants.VIRTUAL_KEY);go.run();});return v;
  }
  void loadApps(){
    all.clear();
    try{for(UserHandle user:launcher.getProfiles())for(LauncherActivityInfo a:launcher.getActivityList(null,user)){
      if(!a.getComponentName().getPackageName().equals(getPackageName()))all.add(new Item(a));
    }}catch(Exception ignored){}
    Collator col=Collator.getInstance();Collections.sort(all,(a,b)->col.compare(a.name,b.name));
  }
  Item byId(String id){for(Item a:all)if(a.id.equals(id))return a;return null;}
  Item byPackage(String... pkgs){for(String pkg:pkgs)for(Item a:all)if(a.pkg.equals(pkg))return a;return null;}
  void loadPins(){
    favorites.clear();dock.clear();
    if(prefs.contains("favorites")){Collections.addAll(favorites,prefs.getString("favorites","").split("\\|"));favorites.remove("");}
    else{
      for(String pkg:new String[]{"com.samsung.android.gallery","com.google.android.apps.photos","com.spotify.music","com.google.android.youtube","com.sec.android.app.myfiles","com.android.settings","com.android.vending","com.google.android.gm"}){
        Item i=byPackage(pkg);if(i!=null&&!favorites.contains(i.id))favorites.add(i.id);
      }
      for(Item i:all)if(favorites.size()<8&&!favorites.contains(i.id))favorites.add(i.id);
    }
    if(prefs.contains("dock")){Collections.addAll(dock,prefs.getString("dock","").split("\\|"));dock.remove("");}
    else{
      for(String[] picks:new String[][]{{"com.samsung.android.dialer","com.google.android.dialer"},{"com.google.android.apps.messaging","com.samsung.android.messaging"},{"com.sec.android.app.camera"},{"com.android.chrome","org.mozilla.firefox"}}){
        Item i=byPackage(picks);if(i!=null&&!dock.contains(i.id))dock.add(i.id);
      }
      for(Item i:all)if(dock.size()<4&&!dock.contains(i.id))dock.add(i.id);
    }
  }
  void savePins(){prefs.edit().putString("favorites",TextUtils.join("|",favorites)).putString("dock",TextUtils.join("|",dock)).apply();}
  void render(){
    drawer=false;overlay=null;root=new FrameLayout(this);
    root.addView(new Sky(this),new FrameLayout.LayoutParams(-1,-1));
    setContentView(root);
    LinearLayout layout=vertical();root.addView(layout,new FrameLayout.LayoutParams(-1,-1));
    ScrollView scroll=new ScrollView(this);scroll.setFillViewport(false);scroll.setVerticalScrollBarEnabled(false);
    layout.addView(scroll,new LinearLayout.LayoutParams(-1,0,1));
    LinearLayout content=vertical();content.setPadding(d(22),d(12),d(22),d(8));scroll.addView(content);
    LinearLayout top=horizontal();
    TextView logo=label("A E R I S",17,ink,true);top.addView(logo,new LinearLayout.LayoutParams(0,d(48),1));
    top.addView(button("⌕",()->openDrawer(true)),margins(48,48,0,0,10,0));
    top.addView(button("⚙",this::settings),lp(48,48));
    content.addView(top,lp(-1,62));
    TextView welcome=label("A BRIGHTER DIGITAL WORLD",10,secondary,true);welcome.setLetterSpacing(.22f);
    content.addView(welcome,margins(-1,27,0,19,0,1));
    clock=label("",68,ink,false);clock.setTypeface(Typeface.create("sans-serif-light",Typeface.NORMAL));content.addView(clock,lp(-1,86));
    date=label("",17,secondary,false);content.addView(date,lp(-1,28));
    LinearLayout info=vertical();info.setPadding(d(18),d(16),d(18),d(12));info.setBackground(glass(28,false));info.setElevation(d(4));
    LinearLayout line=horizontal();TextView title=label("TODAY AT A GLANCE",11,secondary,true);title.setLetterSpacing(.13f);line.addView(title,new LinearLayout.LayoutParams(0,d(33),1));
    bat=label("",22,accent,true);bat.setGravity(Gravity.RIGHT|Gravity.CENTER_VERTICAL);line.addView(bat,lp(90,33));info.addView(line,lp(-1,38));
    TextView sub=label("Your space, shaped around you.",17,ink,true);info.addView(sub,lp(-1,36));
    TextView lower=label("Swipe into your full app library below.",13,secondary,false);info.addView(lower,lp(-1,25));
    content.addView(info,margins(-1,136,0,24,0,24));
    LinearLayout heading=horizontal();TextView head=label("YOUR SPACE",12,ink,true);head.setLetterSpacing(.16f);
    heading.addView(head,new LinearLayout.LayoutParams(0,d(36),1));
    TextView edit=label("EDIT  +",13,accent,true);edit.setGravity(Gravity.CENTER);edit.setOnClickListener(v->openDrawer(false));heading.addView(edit,lp(70,40));
    content.addView(heading,lp(-1,44));
    GridLayout grid=new GridLayout(this);grid.setColumnCount(4);
    int width=(getResources().getDisplayMetrics().widthPixels-d(44))/4;
    int k=0;
    for(String id:new ArrayList<>(favorites)){
      Item i=byId(id);if(i==null)continue;
      View tile=tile(i,false);
      GridLayout.LayoutParams p=new GridLayout.LayoutParams(GridLayout.spec(k/4),GridLayout.spec(k%4));p.width=width;p.height=d(105);
      grid.addView(tile,p);k++;
    }
    content.addView(grid,lp(-1,-2));
    TextView note=label("LONG PRESS AN APP TO CUSTOMISE",10,secondary,true);note.setGravity(Gravity.CENTER);note.setLetterSpacing(.05f);
    content.addView(note,margins(-1,44,0,6,0,2));
    LinearLayout dockCard=horizontal();dockCard.setPadding(d(6),d(6),d(6),d(6));dockCard.setBackground(glass(28,true));dockCard.setElevation(d(10));
    for(String id:new ArrayList<>(dock)){
      Item a=byId(id);if(a!=null)dockCard.addView(tile(a,true),new LinearLayout.LayoutParams(0,d(70),1));
    }
    dockCard.addView(button("▦",()->openDrawer(false)),new LinearLayout.LayoutParams(0,d(62),1));
    LinearLayout bottom=vertical();bottom.setPadding(d(12),d(4),d(12),d(12));bottom.addView(dockCard,lp(-1,82));
    layout.addView(bottom,lp(-1,98));
    refreshClock();
    try{Intent b=registerReceiver(null,new IntentFilter(Intent.ACTION_BATTERY_CHANGED));int level=b.getIntExtra(BatteryManager.EXTRA_LEVEL,-1);if(level>=0)bat.setText(level+"%");}catch(Exception ignored){}
  }
  View tile(Item a,boolean small){
    LinearLayout v=vertical();v.setGravity(Gravity.TOP|Gravity.CENTER_HORIZONTAL);
    FrameLayout frame=new FrameLayout(this);frame.setBackground(glass(20,false));frame.setElevation(d(2));
    ImageView img=new ImageView(this);img.setImageDrawable(a.icon);img.setScaleType(ImageView.ScaleType.FIT_CENTER);
    FrameLayout.LayoutParams p=new FrameLayout.LayoutParams(d(small?34:40),d(small?34:40),Gravity.CENTER);
    frame.addView(img,p);v.addView(frame,lp(small?54:62,small?54:62));
    if(!small){TextView t=label(a.name,11,ink,true);t.setGravity(Gravity.TOP|Gravity.CENTER_HORIZONTAL);t.setSingleLine();t.setEllipsize(TextUtils.TruncateAt.END);t.setPadding(d(1),d(6),d(1),0);v.addView(t,lp(-1,32));}
    v.setOnClickListener(w->startApp(a));v.setOnLongClickListener(w->{actions(a);return true;});
    v.setContentDescription(a.name);return v;
  }
  void startApp(Item a){
    if(drawer)closeDrawer();
    try{launcher.startMainActivity(a.info.getComponentName(),a.info.getUser(),null,null);}
    catch(Exception e){try{startActivity(new Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER).setComponent(a.info.getComponentName()).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));}catch(Exception ex){Toast.makeText(this,"App unavailable",Toast.LENGTH_SHORT).show();}}
  }
  void actions(Item a){
    boolean f=favorites.contains(a.id),t=dock.contains(a.id);
    String[] names={f?"Remove favourite":"Add favourite",t?"Remove from dock":"Add to dock","App information"};
    new AlertDialog.Builder(this).setTitle(a.name).setItems(names,(dialog,choice)->{
      if(choice==0){if(f)favorites.remove(a.id);else if(favorites.size()<8)favorites.add(a.id);else Toast.makeText(this,"Eight favourites maximum",0).show();}
      else if(choice==1){if(t)dock.remove(a.id);else if(dock.size()<4)dock.add(a.id);else{new AlertDialog.Builder(this).setTitle("Replace dock position").setItems(new String[]{"1","2","3","4"},(j,index)->{dock.set(index,a.id);savePins();render();}).show();return;}}
      else{startActivity(new Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS,android.net.Uri.parse("package:"+a.pkg)));return;}
      savePins();render();
    }).show();
  }
  void openDrawer(boolean focus){
    if(drawer)return;drawer=true;
    overlay=new FrameLayout(this);root.addView(overlay,new FrameLayout.LayoutParams(-1,-1));
    View dim=new View(this);dim.setBackgroundColor(0x5181c9d5);overlay.addView(dim,new FrameLayout.LayoutParams(-1,-1));dim.setOnClickListener(v->closeDrawer());
    LinearLayout panel=vertical();panel.setBackground(glass(27,true));panel.setPadding(d(15),d(18),d(15),d(10));panel.setElevation(d(14));
    FrameLayout.LayoutParams outer=new FrameLayout.LayoutParams(-1,(int)(getResources().getDisplayMetrics().heightPixels*.85),Gravity.BOTTOM);outer.setMargins(d(6),0,d(6),d(2));overlay.addView(panel,outer);
    LinearLayout bar=horizontal();TextView title=label("APPLICATION LIBRARY",17,ink,true);bar.addView(title,new LinearLayout.LayoutParams(0,d(52),1));
    bar.addView(button("×",this::closeDrawer),lp(47,47));panel.addView(bar,margins(-1,52,3,0,3,10));
    search=new EditText(this);search.setTextSize(16);search.setSingleLine();search.setHint("Search installed applications");search.setTextColor(ink);search.setHintTextColor(secondary);
    search.setPadding(d(18),0,d(12),0);search.setBackground(glass(17,false));panel.addView(search,margins(-1,54,2,0,2,14));
    TextView hint=label(all.size()+" APPS  ·  HOLD AN ICON TO PIN IT",11,secondary,true);panel.addView(hint,margins(-1,27,8,0,0,7));
    drawerGrid=new GridView(this);drawerGrid.setNumColumns(4);drawerGrid.setVerticalSpacing(d(11));drawerGrid.setHorizontalSpacing(d(4));drawerGrid.setSelector(android.R.color.transparent);
    drawerGrid.setClipToPadding(false);drawerGrid.setPadding(d(1),d(4),d(1),d(15));panel.addView(drawerGrid,new LinearLayout.LayoutParams(-1,0,1));
    filtered.clear();filtered.addAll(all);
    adapter=new BaseAdapter(){
      public int getCount(){return filtered.size();}
      public Object getItem(int p){return filtered.get(p);}
      public long getItemId(int p){return p;}
      public View getView(int p,View convert,android.view.ViewGroup parent){return tile(filtered.get(p),false);}
    };
    drawerGrid.setAdapter(adapter);
    search.addTextChangedListener(new TextWatcher(){
      public void beforeTextChanged(CharSequence s,int st,int c,int a){}
      public void onTextChanged(CharSequence s,int st,int before,int count){
        String query=s.toString().toLowerCase(Locale.ROOT);filtered.clear();
        for(Item item:all)if(item.name.toLowerCase(Locale.ROOT).contains(query))filtered.add(item);
        adapter.notifyDataSetChanged();
      }
      public void afterTextChanged(Editable e){}
    });
    panel.setTranslationY(d(55));panel.setAlpha(0);panel.animate().translationY(0).alpha(1).setDuration(180).start();
    if(focus){search.requestFocus();search.postDelayed(()->{((InputMethodManager)getSystemService(INPUT_METHOD_SERVICE)).showSoftInput(search,InputMethodManager.SHOW_IMPLICIT);},180);}
  }
  void closeDrawer(){if(overlay!=null&&overlay.getParent()!=null)root.removeView(overlay);overlay=null;drawer=false;}
  void settings(){
    String[] themes={"Lagoon · crystal blue","Sunrise · vivid orange","Garden · fresh mint","Android settings","Choose default home"};
    new AlertDialog.Builder(this).setTitle("Personalise AERIS").setItems(themes,(di,n)->{
      if(n<3){theme=n;accent=COLORS[n];prefs.edit().putInt("theme",theme).apply();render();}
      else if(n==3)startActivity(new Intent(Settings.ACTION_SETTINGS));
      else requestHome();
    }).show();
  }
  void refreshClock(){
    Date now=new Date();if(clock!=null)clock.setText(new SimpleDateFormat(DateFormat.is24HourFormat(this)?"HH:mm":"h:mm",Locale.getDefault()).format(now));
    if(date!=null)date.setText(new SimpleDateFormat("EEEE, d MMMM",Locale.getDefault()).format(now));
  }
  void requestHome(){
    if(prefs.getBoolean("asked_home",false))return;
    try{
      RoleManager role=(RoleManager)getSystemService(Context.ROLE_SERVICE);
      if(role!=null&&!role.isRoleHeld(RoleManager.ROLE_HOME)){
        prefs.edit().putBoolean("asked_home",true).apply();
        new AlertDialog.Builder(this).setTitle("Make AERIS your launcher?")
          .setMessage("You can switch home apps at any time in Android settings.")
          .setPositiveButton("Choose AERIS",(dlg,w)->{try{startActivity(role.createRequestRoleIntent(RoleManager.ROLE_HOME));}catch(Exception ignored){}})
          .setNegativeButton("Not now",null).show();
      }
    }catch(Exception ignored){}
  }
  class Sky extends View{
    Paint p=new Paint(3);
    Sky(Context c){super(c);setLayerType(View.LAYER_TYPE_HARDWARE,null);}
    @Override protected void onDraw(Canvas c){
      float w=getWidth(),h=getHeight();p.setShader(new LinearGradient(0,0,w*.5f,h,TOP[theme],BOT[theme],Shader.TileMode.CLAMP));c.drawRect(0,0,w,h,p);p.setShader(null);
      p.setShader(new RadialGradient(w*.84f,h*.18f,w*.75f,new int[]{0xd5ffffff,0x58ffffff,0x00ffffff},null,Shader.TileMode.CLAMP));
      c.drawCircle(w*.84f,h*.18f,w*.75f,p);p.setShader(null);
      for(int i=0;i<3;i++){
        float y=h*(.67f+i*.11f);Path a=new Path();a.moveTo(0,y);a.cubicTo(w*.32f,y-h*.12f,w*.52f,y+h*.1f,w,y-h*.04f);a.lineTo(w,h);a.lineTo(0,h);a.close();
        p.setColor(i==1?0x3973d5d5:0x48ffffff);c.drawPath(a,p);
      }
      p.setColor(0x70ffffff);c.drawCircle(w*.2f,h*.15f,w*.065f,p);p.setColor(0x35ffffff);c.drawCircle(w*.23f,h*.125f,w*.029f,p);
    }
  }
}
