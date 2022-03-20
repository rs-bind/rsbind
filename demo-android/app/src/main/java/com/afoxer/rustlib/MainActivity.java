package com.afoxer.rustlib;

import android.app.Activity;
import android.os.Bundle;
import android.util.Log;

import com.afoxer.xxx.ffi.*;


public class MainActivity extends Activity {
    private static final String TAG = "MainActivity";
    private static DemoTrait demoTrait;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        demoTrait = RustLib.newDemoTrait();

        demoTrait.setup();
        demoTrait.testArgCallback16(new DemoCallback() {
            @Override
            public byte testU81(byte arg, byte arg2) {
                Log.e(TAG, "testU81 -> arg: " + arg + " arg2: " + arg2);
                return 1;
            }

            @Override
            public byte testI82(byte arg, byte arg2) {
                Log.e(TAG, "testI82 -> arg: " + arg + " arg2: " + arg2);
                return 2;
            }

            @Override
            public short testI163(short arg, short arg2) {
                Log.e(TAG, "testI163 -> arg: " + arg + " arg2: " + arg2);
                return 3;
            }

            @Override
            public short testU164(short arg, short arg2) {
                Log.e(TAG, "testU164 -> arg: " + arg + " arg2: " + arg2);
                return 4;
            }

            @Override
            public int testI325(int arg, int arg2) {
                Log.e(TAG, "testI325 -> arg: " + arg + " arg2: " + arg2);
                return 5;
            }

            @Override
            public int testU326(int arg, int arg2) {
                Log.e(TAG, "testU326 -> arg: " + arg + " arg2: " + arg2);
                return 6;
            }

            @Override
            public boolean testBoolFalse(boolean argTrue, boolean argFalse) {
                Log.e(TAG, "testBoolFalse -> argTrue: " + argTrue + " argFalse: " + argFalse);
                return false;
            }

            @Override
            public float testF3230(float arg, float arg2) {
                Log.e(TAG, "testF3230 -> argTrue: " + arg + " argFalse: " + arg2);
                return 30.0f;
            }

            @Override
            public double testF6431(double arg, double arg2) {
                Log.e(TAG, "testF6431 -> argTrue: " + arg + " argFalse: " + arg2);
                return 31.0;
            }

            @Override
            public long testI647(long arg, long arg2) {
                Log.e(TAG, "testI647 -> arg: " + arg + " arg2: " + arg2);
                return 7;
            }

            @Override
            public long testU647(long arg, long arg2) {
                Log.e(TAG, "testU647 -> arg: " + arg + " arg2: " + arg2);
                return 7;            }

            @Override
            public String testStr(String arg) {
                Log.e(TAG, "testStr -> arg: " + arg);
                return "Hello world";
            }

            @Override
            public int testArgVecStr18(String[] arg) {
                Log.e(TAG, "testArgVecStr18 -> arg: " + arg[0]);
                return 18;
            }

            @Override
            public int testArgVecU87(byte[] arg) {
                Log.e(TAG, "testArgVecU87 -> arg: " + arg[0]);
                return 7;
            }

            @Override
            public int testArgVecI88(byte[] arg) {
                Log.e(TAG, "testArgVecI88 -> arg: " + arg[0]);
                return 8;
            }

            @Override
            public int testArgVecI169(Short[] arg) {
                Log.e(TAG, "testArgVecI88 -> arg: " + arg[0]);
                return 9;
            }

            @Override
            public int testArgVecU1610(Short[] arg) {
                Log.e(TAG, "testArgVecU1610 -> arg: " + arg[0]);
                return 10;
            }

            @Override
            public int testArgVecI3211(Integer[] arg) {
                Log.e(TAG, "testArgVecI3211 -> arg: " + arg[0]);
                return 11;
            }

            @Override
            public int testArgVecU3212(Integer[] arg) {
                Log.e(TAG, "testArgVecU3212 -> arg: " + arg[0]);
                return 12;
            }

            @Override
            public long testArgVecI6411(Long[] arg) {
                Log.e(TAG, "testArgVecI6411 -> arg: " + arg[0]);
                return 11;            }

            @Override
            public long testArgVecU6412(Long[] arg) {
                Log.e(TAG, "testArgVecU6412 -> arg: " + arg[0]);
                return 12;
            }

            @Override
            public boolean testArgVecBoolTrue(Boolean[] argTrue) {
                Log.e(TAG, "testArgVecBoolTrue -> argTrue: " + argTrue[0]);
                return true;
            }

            @Override
            public int testArgVecStruct17(DemoStruct[] arg) {
                Log.e(TAG, "testArgVecStruct17 -> arg: " + arg[0]);
                return 17;
            }

            @Override
            public int testTwoVecArg13(Integer[] arg, Integer[] arg1) {
                Log.e(TAG, "testTwoVecArg13 -> arg: " + arg[0]);
                return 13;
            }

            @Override
            public byte[] testReturnVecU8() {
                Log.e(TAG, "testReturnVecU8");
                return new byte[]{100};
            }

            @Override
            public byte[] testReturnVecI8() {
                Log.e(TAG, "testReturnVecI8");
                return new byte[]{100};
            }

            @Override
            public Short[] testReturnVecI16() {
                Log.e(TAG, "testReturnVecI16");
                return new Short[]{100};
            }

            @Override
            public Short[] testReturnVecU16() {
                Log.e(TAG, "testReturnVecU16");
                return new Short[]{100};
            }

            @Override
            public Integer[] testReturnVecI32() {
                Log.e(TAG, "testReturnVecI32");
                return new Integer[]{100};
            }

            @Override
            public Integer[] testReturnVecU32() {
                Log.e(TAG, "testReturnVecU32");
                return new Integer[]{100};
            }

            @Override
            public Long[] testReturnVecI64() {
                Log.e(TAG, "testReturnVecI64");
                return new Long[]{100L};
            }

            @Override
            public Long[] testReturnVecU64() {
                Log.e(TAG, "testReturnVecU64");
                return new Long[]{100L};
            }

            @Override
            public byte[] testTwoVecU8(byte[] input) {
                Log.e(TAG, "testReturnVecU64 input = " + input[0]);
                return new byte[]{100};
            }

            @Override
            public int testArgStruct14(DemoStruct arg) {
                Log.e(TAG, "testArgStruct14 -> arg: " + arg);
                return 14;
            }

            @Override
            public int testTwoArgStruct15(DemoStruct arg, DemoStruct arg1) {
                Log.e(TAG, "testTwoArgStruct15 -> arg: " + arg + " arg1: " + arg1);
                return 15;
            }

            @Override
            public void testNoReturn() {
                Log.e(TAG, "testNoReturn");
            }
        });
    }
}
